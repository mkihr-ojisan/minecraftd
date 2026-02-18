use std::{fmt::Display, path::Path, time::Duration};

use anyhow::{Context, bail};
use clap::Parser;
use mcctl_protocol::{ConnectionType, ExtensionInfo, ExtensionType, ServerStatus, client::Client};
use minecraftd_manifest::ServerManifest;
use nix::sys::termios::{LocalFlags, SetArg, tcgetattr, tcsetattr};
use terminal_size::{Height, Width, terminal_size};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::cli::{Extensions, ExtensionsAddArgs, Subcommand};

mod cli;
mod eula;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = start().await {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

async fn start() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    match args.command {
        Subcommand::Create(args) => {
            create_server(args).await?;
        }
        Subcommand::Start(args) => {
            start_server(args).await?;
        }
        Subcommand::Stop(args) => {
            stop_server(args).await?;
        }
        Subcommand::Restart(args) => {
            restart_server(args).await?;
        }
        Subcommand::Kill(args) => {
            kill_server(args).await?;
        }
        Subcommand::Attach(args) => {
            attach_server_terminal(args).await?;
        }
        Subcommand::Update(args) => {
            update_server(args).await?;
        }
        Subcommand::Ps => {
            list_running_servers().await?;
        }
        Subcommand::Extensions { command } => match command {
            Extensions::Add(args) => {
                add_extension(args).await?;
            }
        },
    }

    Ok(())
}

async fn create_server(args: cli::CreateArgs) -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };
    std::fs::create_dir_all(&server_dir).with_context(|| {
        format!(
            "Failed to create directory at path '{}'",
            server_dir.display()
        )
    })?;
    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    let name = match args.name {
        Some(n) => n,
        None => inquire::Text::new("Server name:")
            .with_default(
                Path::new(&server_dir)
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_else(|| "A Minecraft Server".into())
                    .as_ref(),
            )
            .prompt()?,
    };

    let server_implementation = match args.server_implementation {
        Some(name) => name,
        None => {
            let server_implementations = client
                .get_server_implementations()
                .await
                .context("Failed to get server implementations")?;

            inquire::Select::new("Server implementation:", server_implementations).prompt()?
        }
    };

    let version = match args.version {
        Some(v) => v,
        None => {
            struct VersionDisplay(mcctl_protocol::Version);
            impl Display for VersionDisplay {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    if self.0.is_stable {
                        write!(f, "{}", self.0.name)
                    } else {
                        write!(f, "{} (unstable)", self.0.name)
                    }
                }
            }

            let versions = client
                .get_versions(&server_implementation)
                .await
                .context("Failed to get versions")?;

            if versions.len() == 1 {
                versions.into_iter().next().unwrap().name
            } else {
                let versions = versions.into_iter().map(VersionDisplay).collect::<Vec<_>>();

                let latest_stable_index = versions.iter().position(|v| v.0.is_stable).unwrap_or(0);

                inquire::Select::new("Version:", versions)
                    .with_starting_cursor(latest_stable_index)
                    .prompt()?
                    .0
                    .name
            }
        }
    };

    let build = match args.build {
        Some(b) => b,
        None => {
            let builds = client
                .get_builds(&server_implementation, &version)
                .await
                .context("Failed to get builds")?;

            if builds.len() == 1 {
                builds.into_iter().next().unwrap().name
            } else {
                struct BuildDisplay(mcctl_protocol::Build);
                impl Display for BuildDisplay {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        if self.0.is_stable {
                            write!(f, "{}", self.0.name)
                        } else {
                            write!(f, "{} (unstable)", self.0.name)
                        }
                    }
                }

                let builds = builds.into_iter().map(BuildDisplay).collect::<Vec<_>>();

                let latest_stable_index = builds.iter().position(|b| b.0.is_stable).unwrap_or(0);

                inquire::Select::new("Build:", builds)
                    .with_starting_cursor(latest_stable_index)
                    .prompt()?
                    .0
                    .name
            }
        }
    };

    let connection = match &args.connection {
        Some(c) => c,
        None => {
            let connections = vec!["direct", "proxy"];
            inquire::Select::new("Connection type:", connections).prompt()?
        }
    };
    let connection = match connection {
        "direct" => ConnectionType::Direct,
        "proxy" => ConnectionType::Proxy,
        _ => bail!("Invalid connection type '{}'", connection),
    };

    let hostname = match args.hostname {
        Some(h) => Some(h),
        None => {
            if connection == ConnectionType::Proxy {
                Some(inquire::Text::new("Proxy hostname:").prompt()?)
            } else {
                None
            }
        }
    };

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Creating server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    client
        .create_server(
            server_dir,
            name,
            server_implementation,
            version,
            build,
            connection,
            hostname,
        )
        .await?;

    pb.finish_with_message("Server created successfully.");

    Ok(())
}

async fn start_server(args: cli::StartArgs) -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    if !eula::is_accepted(&server_dir).await? {
        let accept = inquire::Confirm::new(
            "You must accept the EULA to start the server. Do you accept the EULA?",
        )
        .with_default(false)
        .prompt()?;

        if !accept {
            bail!("EULA not accepted. Aborting.");
        }

        eula::accept(&server_dir).await?;
    }

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Starting server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    client.start_server(&server_dir).await?;

    client.wait_server_ready(server_dir).await?;

    pb.finish_with_message("Server started successfully.");

    Ok(())
}

async fn stop_server(args: cli::StopArgs) -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Stopping server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    client.stop_server(server_dir).await?;

    pb.finish_with_message("Server stopped successfully.");

    Ok(())
}

async fn restart_server(args: cli::RestartArgs) -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Restarting server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    client.restart_server(&server_dir).await?;

    client.wait_server_ready(server_dir).await?;

    pb.finish_with_message("Server restarted successfully.");

    Ok(())
}

async fn kill_server(args: cli::KillArgs) -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Killing server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    client.kill_server(server_dir).await?;

    pb.finish_with_message("Server killed successfully.");

    Ok(())
}

async fn attach_server_terminal(args: cli::AttachArgs) -> anyhow::Result<()> {
    let client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    let (mut terminal_reader, mut terminal_writer) = client.attach_terminal(server_dir).await?;

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    let mut attr = tcgetattr(&stdin).context("Failed to get terminal attributes")?;
    let orig_attr = attr.clone();
    attr.local_flags
        .remove(LocalFlags::ICANON | LocalFlags::ECHO);
    tcsetattr(&stdin, SetArg::TCSANOW, &attr).context("Failed to set terminal attributes")?;

    let (exit_signal_tx, mut exit_signal_rx) = tokio::sync::mpsc::channel::<()>(1);

    let input_task = tokio::spawn({
        async move {
            let result: anyhow::Result<()> = async {
                let mut buffer = [0u8; 1024];
                let mut sigwinch =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::window_change())
                        .context("Failed to create SIGWINCH signal stream")?;

                loop {
                    tokio::select! {
                        result = stdin.read(&mut buffer) => {
                            let n = result.context("Failed to read from stdin")?;
                            if n == 0 {
                                break;
                            }
                            terminal_writer
                                .write(buffer[..n].to_vec())
                                .await
                                .context("Failed to send terminal input")?;
                        }
                        _ = sigwinch.recv() => {
                            let (Width(cols), Height(rows)) =
                                terminal_size().context("Failed to get terminal size")?;
                            terminal_writer
                                .resize(cols as u32, rows as u32)
                                .await
                                .context("Failed to send resize command")?;
                        }
                        _ = exit_signal_rx.recv() => {
                            break;
                        }
                    };
                }
                Ok(())
            }
            .await;

            if let Err(e) = result {
                eprintln!("Error in input task: {:?}", e);
            }
        }
    });

    let output_task = tokio::spawn({
        async move {
            let result: anyhow::Result<()> = async {
                loop {
                    let Some(output) = terminal_reader
                        .read()
                        .await
                        .context("Failed to read from terminal")?
                    else {
                        exit_signal_tx.send(()).await.ok();
                        break Ok(());
                    };

                    stdout
                        .write_all(&output.content)
                        .await
                        .context("Failed to write to stdout")?;
                    stdout.flush().await.context("Failed to flush stdout")?;
                }
            }
            .await;

            if let Err(e) = result {
                eprintln!("Error in output task: {:?}", e);
            }
        }
    });

    tokio::try_join!(input_task, output_task).unwrap();

    tcsetattr(tokio::io::stdin(), SetArg::TCSANOW, &orig_attr)
        .context("Failed to restore terminal attributes")?;

    Ok(())
}

async fn update_server(args: cli::UpdateArgs) -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Updating server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let update_type = match args.update_type {
        cli::UpdateType::Stable => mcctl_protocol::UpdateType::Stable,
        cli::UpdateType::Latest => mcctl_protocol::UpdateType::Latest,
    };

    let result = client.update_server(&server_dir, update_type).await?;

    if result.updated {
        pb.finish_with_message(format!(
            "Server updated successfully from version {} build {} to version {} build {}.",
            result.old_version.unwrap_or_default(),
            result.old_build.unwrap_or_default(),
            result.new_version.unwrap_or_default(),
            result.new_build.unwrap_or_default()
        ));
    } else {
        pb.finish_with_message("Server is already up to date.");
    }

    Ok(())
}

async fn list_running_servers() -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let servers = client.get_running_servers().await?;

    let mut table = Vec::<[String; 6]>::new();
    table.push([
        "NAME".to_string(),
        "STATUS".to_string(),
        "UPTIME".to_string(),
        "PORT".to_string(),
        "PLAYERS".to_string(),
        "DIRECTORY".to_string(),
    ]);

    for server in servers {
        table.push([
            server.name,
            match ServerStatus::try_from(server.status) {
                Ok(ServerStatus::Starting) => "Starting",
                Ok(ServerStatus::Ready) => "Ready",
                Ok(ServerStatus::Stopping) => "Stopping",
                Ok(ServerStatus::Restarting) => "Restarting",
                Err(_) => "Unknown",
            }
            .to_string(),
            match Duration::from_secs(server.uptime_seconds) {
                d if d.as_secs() < 60 => format!("{}s", d.as_secs()),
                d if d.as_secs() < 3600 => format!("{}m{}s", d.as_secs() / 60, d.as_secs() % 60),
                d => format!(
                    "{}h{}m{}s",
                    d.as_secs() / 3600,
                    (d.as_secs() % 3600) / 60,
                    d.as_secs() % 60
                ),
            },
            server.port.to_string(),
            if let Some(player_count) = server.player_count
                && let Some(max_players) = server.max_players
            {
                format!("{}/{}", player_count, max_players)
            } else {
                "-".to_string()
            },
            server.server_dir.to_string(),
        ]);
    }

    let column_widths = (0..6)
        .map(|i| table.iter().map(|row| row[i].len()).max().unwrap_or(0))
        .collect::<Vec<_>>();

    for row in table {
        for (i, cell) in row.iter().enumerate() {
            print!("{:width$}  ", cell, width = column_widths[i]);
        }
        println!();
    }

    Ok(())
}

async fn add_extension(args: ExtensionsAddArgs) -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    let manifest = ServerManifest::load(&server_dir)
        .await
        .context("Failed to load server manifest")?;

    let (provider, type_, extension) = if let Some(url) = args.url {
        let result = client.get_extension_id_by_url(&url).await?;

        let type_ = ExtensionType::try_from(result.r#type).context("Invalid extension type")?;

        (
            result.provider,
            type_,
            ExtensionInfo {
                id: result.extension_id,
                name: url,
            },
        )
    } else {
        let providers = client.get_extension_providers().await?;
        let provider = inquire::Select::new("Extension provider:", providers).prompt()?;

        let type_ = match inquire::Select::new("Extension type:", vec!["mod", "plugin"]).prompt()? {
            "mod" => mcctl_protocol::ExtensionType::Mod,
            "plugin" => mcctl_protocol::ExtensionType::Plugin,
            _ => bail!("Invalid extension type"),
        };

        let search_query = inquire::Text::new("Search query:").prompt()?;

        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_message("Searching for extensions...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let extensions = client
            .search_extension(
                &provider,
                type_,
                &manifest.version,
                &search_query,
                args.allow_incompatible_versions,
            )
            .await
            .context("Failed to search for extensions")?;

        pb.finish_and_clear();

        struct ExtensionDisplay(mcctl_protocol::ExtensionInfo);
        impl Display for ExtensionDisplay {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0.name)
            }
        }

        let extension = inquire::Select::new(
            "Select a extension:",
            extensions.into_iter().map(ExtensionDisplay).collect(),
        )
        .prompt()?
        .0;

        (provider, type_, extension)
    };

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Fetching extension versions...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let extension_versions = client
        .get_extension_versions(
            &provider,
            type_,
            &manifest.version,
            &extension.id,
            args.allow_incompatible_versions,
        )
        .await
        .context("Failed to get extension versions")?;

    pb.finish_and_clear();

    struct ExtensionVersionDisplay(mcctl_protocol::ExtensionVersionInfo);
    impl Display for ExtensionVersionDisplay {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.0.is_stable {
                write!(f, "{}", self.0.version)
            } else {
                write!(f, "{} (unstable)", self.0.version)
            }
        }
    }

    let latest_stable_index = extension_versions
        .iter()
        .position(|v| v.is_stable)
        .unwrap_or(0);

    let extension_version = inquire::Select::new(
        "Select a extension version:",
        extension_versions
            .into_iter()
            .map(ExtensionVersionDisplay)
            .collect(),
    )
    .with_starting_cursor(latest_stable_index)
    .prompt()?;

    let auto_update = if let Some(auto_update) = args.auto_update {
        auto_update
    } else {
        inquire::Confirm::new("Enable auto-updates for this extension?")
            .with_default(false)
            .prompt()?
    };

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Adding extension to server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();
    let result = client
        .add_extension(
            &server_dir,
            &provider,
            type_,
            &extension.id,
            &extension_version.0.id,
            args.allow_incompatible_versions,
            auto_update,
        )
        .await
        .context("Failed to add extension to server")?;

    pb.finish_with_message("Extension added successfully. Added extensions:");
    for extension in result.added_extensions {
        println!("  - {}", extension.name);
    }

    Ok(())
}
