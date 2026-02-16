use std::path::Path;

use anyhow::{Context, bail};
use minecraftd_manifest::{Connection, ServerManifest};

use crate::server::{implementations::get_server_implementation, java_runtime::JavaRuntimeExt};

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

    // download the server jar if it is not already cached
    let _server_jar_path = implementation
        .get_server_jar_path(server_dir, version, build)
        .await
        .context("Failed to prepare server jar")?;

    let default_java_runtime = implementation
        .default_java_runtime(version, build)
        .await
        .context("Failed to determine default Java runtime for server")?;

    default_java_runtime
        .prepare()
        .await
        .context("Failed to prepare Java runtime")?;

    let mut manifest =
        ServerManifest::default(server_implementation, version, build, default_java_runtime);

    manifest.name = name.to_string();
    manifest.connection = connection;

    manifest
        .save(server_dir)
        .await
        .context("Failed to save server manifest")?;

    Ok(())
}
