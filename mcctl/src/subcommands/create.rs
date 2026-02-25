use std::{fmt::Display, path::Path, time::Duration};

use anyhow::{Context, bail};
use mcctl_protocol::{ConnectionType, client::Client};

use crate::cli::CreateArgs;

pub async fn create(args: CreateArgs) -> anyhow::Result<()> {
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
