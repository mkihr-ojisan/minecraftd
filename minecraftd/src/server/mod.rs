use std::path::Path;

use anyhow::bail;
use minecraftd_manifest::Connection;

use crate::server::implementations::get_server_implementation;

pub mod config;
mod implementations;
mod java_runtime;
pub mod proxy_server;
pub mod runner;
mod server_list_ping;
mod server_properties;

pub fn get_server_implementations() -> impl Iterator<Item = &'static str> {
    implementations::SERVER_IMPLEMENTATIONS
        .iter()
        .map(|impl_| impl_.name())
}

pub async fn get_server_versions(server_implementation: &str) -> anyhow::Result<Vec<String>> {
    let Some(implementation) = get_server_implementation(server_implementation) else {
        bail!("Unknown server implementation '{}'", server_implementation);
    };

    implementation.get_versions().await
}

pub async fn get_server_builds(
    server_implementation: &str,
    version: &str,
) -> anyhow::Result<Vec<String>> {
    let Some(implementation) = get_server_implementation(server_implementation) else {
        bail!("Unknown server implementation '{}'", server_implementation);
    };

    implementation.get_builds(version).await
}

pub async fn create_server(
    name: &str,
    server_dir: &Path,
    server_implementation: &str,
    version: &str,
    build: &str,
    connection: Connection,
) -> anyhow::Result<()> {
    let Some(implementation) = get_server_implementation(server_implementation) else {
        bail!("Unknown server implementation '{}'", server_implementation);
    };

    tokio::fs::create_dir_all(server_dir).await?;

    let result: anyhow::Result<()> = async {
        let mut default_manifest = implementation
            .create_server(version, build, server_dir)
            .await?;

        default_manifest.name = name.to_string();
        default_manifest.connection = connection;

        default_manifest.save(server_dir).await?;

        Ok(())
    }
    .await;

    if result.is_err() {
        tokio::fs::remove_dir_all(server_dir).await.ok();
    }

    result
}
