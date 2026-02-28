use std::path::Path;

use anyhow::{Context, bail};
use mcctl_protocol::*;
use minecraftd_manifest::{Connection, ServerManifest};

use crate::{
    extension::providers::{EXTENSION_PROVIDERS, get_extension_provider},
    metrics::{self, MetricsQuery},
    runner::{self, TerminalReader, TerminalWriter},
    server,
    server_implementations::{SERVER_IMPLEMENTATIONS, get_server_implementation},
};

struct RequestHandler;

impl mcctl_protocol::server::RequestHandler<anyhow::Error, TerminalReader, TerminalWriter>
    for RequestHandler
{
    async fn get_server_implementations() -> anyhow::Result<Vec<String>> {
        Ok(SERVER_IMPLEMENTATIONS
            .iter()
            .map(|impl_| impl_.name().to_string())
            .collect())
    }

    async fn get_versions(
        server_implementation: &str,
    ) -> anyhow::Result<Vec<mcctl_protocol::Version>> {
        Ok(get_server_implementation(server_implementation)
            .with_context(|| format!("Unknown server implementation '{}'", server_implementation))?
            .get_versions()
            .await?
            .into_iter()
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
        Ok(get_server_implementation(server_implementation)
            .with_context(|| format!("Unknown server implementation '{}'", server_implementation))?
            .get_builds(version)
            .await?
            .into_iter()
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

        runner::start_server(server_dir).await?;

        Ok(())
    }

    async fn stop_server(server_dir: &Path) -> anyhow::Result<()> {
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        runner::stop_server(server_dir).await?;

        Ok(())
    }

    async fn kill_server(server_dir: &Path) -> anyhow::Result<()> {
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        runner::kill_server(server_dir).await?;

        Ok(())
    }

    async fn attach_terminal(
        server_dir: &Path,
    ) -> Result<(TerminalReader, TerminalWriter), anyhow::Error> {
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        runner::attach_terminal(server_dir).await
    }

    async fn get_running_servers() -> anyhow::Result<Vec<RunningServer>> {
        let servers = runner::get_running_servers().await;

        Ok(servers
            .into_iter()
            .map(|s| RunningServer {
                server_dir: s.server_dir.to_string_lossy().to_string(),
                name: s.name,
                status: match s.status {
                    runner::ServerStatus::Starting { restarting: false } => ServerStatus::Starting,
                    runner::ServerStatus::Ready => ServerStatus::Ready,
                    runner::ServerStatus::Stopping { restarting: false } => ServerStatus::Stopping,
                    runner::ServerStatus::Starting { restarting: true }
                    | runner::ServerStatus::Stopping { restarting: true } => {
                        ServerStatus::Restarting
                    }
                    runner::ServerStatus::Stopped => unreachable!(),
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

        runner::wait_ready(server_dir).await?;

        Ok(())
    }

    async fn restart_server(server_dir: &Path) -> anyhow::Result<()> {
        let server_dir = Path::new(&server_dir);
        if !server_dir.is_absolute() {
            bail!("server_dir must be absolute");
        }

        runner::restart_server(server_dir).await?;

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
        Ok(EXTENSION_PROVIDERS
            .iter()
            .map(|provider| provider.name().to_string())
            .collect())
    }

    async fn search_extension(
        provider: &str,
        type_: ExtensionType,
        server_version: &str,
        query: &str,
        include_incompatible_versions: bool,
    ) -> anyhow::Result<Vec<ExtensionInfo>> {
        Ok(get_extension_provider(provider)
            .with_context(|| format!("Unknown extension provider '{}'", provider))?
            .search_extension(
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
            .map(|e| ExtensionInfo {
                id: e.id,
                name: e.name,
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
        Ok(get_extension_provider(provider)
            .with_context(|| format!("Unknown extension provider '{}'", provider))?
            .get_extension_versions(
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
        auto_update: bool,
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
            auto_update,
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
        for provider in EXTENSION_PROVIDERS.iter() {
            if let Ok(info) = provider.get_extension_info_by_url(url).await {
                return Ok(GetExtensionIdByUrlResponse {
                    provider: provider.name().to_string(),
                    r#type: match info.type_ {
                        minecraftd_manifest::ExtensionType::Mod => ExtensionType::Mod,
                        minecraftd_manifest::ExtensionType::Plugin => ExtensionType::Plugin,
                    } as i32,
                    extension_id: info.id,
                });
            }
        }

        Err(anyhow::anyhow!("No extension provider recognized the URL"))
    }

    async fn get_metrics(
        server_dir: &Path,
        metric: String,
        start_timestamp: i64,
        end_timestamp: i64,
        aggregation: Aggregation,
        downsample_interval: Option<i64>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> anyhow::Result<GetMetricsResponse> {
        let manifest = ServerManifest::load(server_dir).await?;

        let query = MetricsQuery {
            server_id: manifest.id,
            metric,
            start_timestamp,
            end_timestamp,
            aggregation: match aggregation {
                Aggregation::None => tsink::Aggregation::None,
                Aggregation::Min => tsink::Aggregation::Min,
                Aggregation::Max => tsink::Aggregation::Max,
                Aggregation::Avg => tsink::Aggregation::Avg,
                Aggregation::Sum => tsink::Aggregation::Sum,
                Aggregation::Last => tsink::Aggregation::Last,
            },
            downsample_interval,
            limit: limit.map(|l| l as usize),
            offset: offset.map(|o| o as usize),
        };

        let data_points = metrics::query(query).await?;

        Ok(GetMetricsResponse {
            data_points: data_points
                .into_iter()
                .map(|dp| mcctl_protocol::MetricDataPoint {
                    timestamp: dp.timestamp,
                    value: dp.value,
                })
                .collect(),
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
