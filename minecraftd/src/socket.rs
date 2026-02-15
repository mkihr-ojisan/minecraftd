use std::path::Path;

use anyhow::{Context, bail};
use mcctl_protocol::*;

use crate::server::{
    self,
    config::Connection,
    runner::{TerminalReader, TerminalWriter},
};

struct RequestHandler;

impl mcctl_protocol::server::RequestHandler<anyhow::Error, TerminalReader, TerminalWriter>
    for RequestHandler
{
    async fn get_server_implementations() -> anyhow::Result<Vec<String>> {
        Ok(server::get_server_implementations()
            .map(String::from)
            .collect())
    }

    async fn get_versions(server_implementation: &str) -> anyhow::Result<Vec<String>> {
        server::get_server_versions(server_implementation).await
    }

    async fn get_builds(server_implementation: &str, version: &str) -> anyhow::Result<Vec<String>> {
        server::get_server_builds(server_implementation, version).await
    }

    async fn create_server(
        name: &str,
        server_dir: &Path,
        server_implementation: &str,
        version: &str,
        build: &str,
        connection: ConnectionType,
        hostname: Option<&str>,
    ) -> anyhow::Result<()> {
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        let connection = match connection {
            mcctl_protocol::ConnectionType::Direct => Connection::Direct,
            mcctl_protocol::ConnectionType::Proxy => {
                let hostname = hostname
                    .as_ref()
                    .context("Hostname must be provided for proxy connection")?;
                Connection::Proxy {
                    hostname: hostname.to_string(),
                }
            }
        };

        server::create_server(
            name,
            server_dir,
            server_implementation,
            version,
            build,
            connection,
        )
        .await?;

        Ok(())
    }

    async fn start_server(server_dir: &Path) -> anyhow::Result<()> {
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        server::runner::start_server(server_dir).await?;

        Ok(())
    }

    async fn stop_server(server_dir: &Path) -> anyhow::Result<()> {
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        server::runner::stop_server(server_dir).await?;

        Ok(())
    }

    async fn kill_server(server_dir: &Path) -> anyhow::Result<()> {
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        server::runner::kill_server(server_dir).await?;

        Ok(())
    }

    async fn attach_terminal(
        server_dir: &Path,
    ) -> Result<(TerminalReader, TerminalWriter), anyhow::Error> {
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        server::runner::attach_terminal(server_dir).await
    }

    async fn get_running_servers() -> anyhow::Result<Vec<RunningServer>> {
        let servers = server::runner::get_running_servers().await;

        Ok(servers
            .into_iter()
            .map(|s| RunningServer {
                server_dir: s.server_dir.to_string_lossy().to_string(),
                name: s.name,
                status: match s.status {
                    server::runner::ServerStatus::Starting { restarting: false } => {
                        ServerStatus::Starting
                    }
                    server::runner::ServerStatus::Ready => ServerStatus::Ready,
                    server::runner::ServerStatus::Stopping { restarting: false } => {
                        ServerStatus::Stopping
                    }
                    server::runner::ServerStatus::Starting { restarting: true }
                    | server::runner::ServerStatus::Stopping { restarting: true } => {
                        ServerStatus::Restarting
                    }
                    server::runner::ServerStatus::Stopped => unreachable!(),
                } as i32,
                uptime_seconds: s.uptime.as_secs(),
                port: s.server_port as u32,
                player_count: s.players.as_ref().map(|p| p.online),
                max_players: s.players.as_ref().map(|p| p.max),
            })
            .collect())
    }

    async fn wait_ready(server_dir: &Path) -> anyhow::Result<()> {
        let server_dir = Path::new(&server_dir);
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        server::runner::wait_ready(server_dir).await?;

        Ok(())
    }

    async fn restart_server(server_dir: &Path) -> anyhow::Result<()> {
        let server_dir = Path::new(&server_dir);
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        server::runner::restart_server(server_dir).await?;

        Ok(())
    }
}

pub async fn start_server() -> anyhow::Result<()> {
    let shutdown_signal = async {
        let sigint = tokio::signal::ctrl_c();
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to set up SIGTERM handler");

        tokio::select! {
            _ = sigint => {
                info!("Received SIGINT, shutting down...");
            },
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down...");
            },
        };
    };

    mcctl_protocol::server::listen::<
        anyhow::Error,
        TerminalReader,
        TerminalWriter,
        RequestHandler,
    >(shutdown_signal)
    .await?;

    Ok(())
}
