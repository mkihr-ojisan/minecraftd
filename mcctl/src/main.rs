use std::{path::Path, time::Duration};

use anyhow::{Context, bail};
use clap::Parser;
use mcctl_protocol::{ConnectionType, ServerStatus, client::Client};
use nix::sys::termios::{LocalFlags, SetArg, tcgetattr, tcsetattr};
use terminal_size::{Height, Width, terminal_size};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::cli::Subcommand;

mod cli;

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
        Subcommand::Ps => {
            list_running_servers().await?;
        }
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
            let versions = client
                .get_versions(&server_implementation)
                .await
                .context("Failed to get versions")?;

            inquire::Select::new("Version:", versions).prompt()?
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
                builds.into_iter().next().unwrap()
            } else {
                inquire::Select::new("Build:", builds).prompt()?
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

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Restarting server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    client.restart_server(server_dir).await?;

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
