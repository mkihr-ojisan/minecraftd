use std::path::Path;

use anyhow::{Context, bail};
use mcctl_protocol::*;
use minecraftd_manifest::Connection;

use crate::server::{
    self,
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

    async fn get_versions(
        server_implementation: &str,
    ) -> anyhow::Result<Vec<mcctl_protocol::Version>> {
        Ok(server::get_server_versions(server_implementation)
            .await
            .into_iter()
            .flatten()
            .map(|v| mcctl_protocol::Version {
                name: v.name,
                is_stable: v.is_stable,
            })
            .collect())
    }

    async fn get_builds(
        server_implementation: &str,
        version: &str,
    ) -> anyhow::Result<Vec<mcctl_protocol::Build>> {
        Ok(server::get_server_builds(server_implementation, version)
            .await
            .into_iter()
            .flatten()
            .map(|b| mcctl_protocol::Build {
                name: b.name,
                is_stable: b.is_stable,
            })
            .collect())
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

    async fn update_server(
        server_dir: &Path,
        update_type: UpdateType,
    ) -> anyhow::Result<UpdateServerResponse> {
        let server_dir = Path::new(&server_dir);
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        let update_type = match update_type {
            UpdateType::Stable => server::UpdateType::Stable,
            UpdateType::Latest => server::UpdateType::Latest,
        };

        let update_result = server::update_server(server_dir, update_type).await?;

        Ok(match update_result {
            server::UpdateServerResult::NoUpdateNeeded => UpdateServerResponse {
                updated: false,
                old_version: None,
                old_build: None,
                new_version: None,
                new_build: None,
            },
            server::UpdateServerResult::Updated {
                old_version,
                old_build,
                new_version,
                new_build,
            } => UpdateServerResponse {
                updated: true,
                old_version: Some(old_version),
                old_build: Some(old_build),
                new_version: Some(new_version),
                new_build: Some(new_build),
            },
        })
    }

    async fn get_extension_providers() -> anyhow::Result<Vec<String>> {
        Ok(server::get_extension_providers()
            .map(String::from)
            .collect())
    }

    async fn search_extension(
        provider: &str,
        type_: ExtensionType,
        server_version: &str,
        query: &str,
        include_incompatible_versions: bool,
    ) -> anyhow::Result<Vec<ExtensionInfo>> {
        Ok(server::search_extensions(
            provider,
            match type_ {
                ExtensionType::Mod => minecraftd_manifest::ExtensionType::Mod,
                ExtensionType::Plugin => minecraftd_manifest::ExtensionType::Plugin,
            },
            server_version,
            query,
            include_incompatible_versions,
        )
        .await?
        .into_iter()
        .map(|m| ExtensionInfo {
            id: m.id,
            name: m.name,
        })
        .collect())
    }

    async fn get_extension_versions(
        provider: &str,
        type_: ExtensionType,
        server_version: &str,
        extension_id: &str,
        include_incompatible_versions: bool,
    ) -> anyhow::Result<Vec<ExtensionVersionInfo>> {
        Ok(server::get_extension_versions(
            provider,
            match type_ {
                ExtensionType::Mod => minecraftd_manifest::ExtensionType::Mod,
                ExtensionType::Plugin => minecraftd_manifest::ExtensionType::Plugin,
            },
            server_version,
            extension_id,
            include_incompatible_versions,
        )
        .await?
        .into_iter()
        .map(|v| ExtensionVersionInfo {
            id: v.id,
            version: v.version,
            is_stable: v.is_stable,
        })
        .collect())
    }

    async fn add_extension(
        server_dir: &Path,
        provider: &str,
        type_: ExtensionType,
        extension_id: &str,
        extension_version_id: &str,
        allow_incompatible_versions: bool,
    ) -> anyhow::Result<AddExtensionResponse> {
        let result = server::add_extension(
            server_dir,
            provider,
            match type_ {
                ExtensionType::Mod => minecraftd_manifest::ExtensionType::Mod,
                ExtensionType::Plugin => minecraftd_manifest::ExtensionType::Plugin,
            },
            extension_id,
            extension_version_id,
            allow_incompatible_versions,
        )
        .await?;

        Ok(AddExtensionResponse {
            added_extensions: result
                .added_extensions
                .into_iter()
                .map(|m| ExtensionInfo {
                    id: m.id,
                    name: m.name,
                })
                .collect(),
        })
    }

    async fn get_extension_id_by_url(url: &str) -> anyhow::Result<GetExtensionIdByUrlResponse> {
        let (provider, info) = server::get_extension_info_by_url(url).await?;

        Ok(GetExtensionIdByUrlResponse {
            r#type: info.type_ as i32,
            provider: provider.to_string(),
            extension_id: info.id,
        })
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
    >(shutdown_signal, |e| format!("{e:?}"))
    .await?;

    Ok(())
}
