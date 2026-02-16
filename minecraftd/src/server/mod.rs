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

    Ok(implementation
        .get_versions()
        .await?
        .into_iter()
        .map(|v| v.name)
        .collect())
}

pub async fn get_server_builds(
    server_implementation: &str,
    version: &str,
) -> anyhow::Result<Vec<String>> {
    let Some(implementation) = get_server_implementation(server_implementation) else {
        bail!("Unknown server implementation '{}'", server_implementation);
    };

    Ok(implementation
        .get_builds(version)
        .await?
        .into_iter()
        .map(|b| b.name)
        .collect())
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

pub enum UpdateType {
    Stable,
    Latest,
}

pub enum UpdateServerResult {
    NoUpdateNeeded,
    Updated {
        old_version: String,
        old_build: String,
        new_version: String,
        new_build: String,
    },
}

pub async fn update_server(
    server_dir: &Path,
    update_type: UpdateType,
) -> anyhow::Result<UpdateServerResult> {
    if runner::is_server_running(server_dir).await? {
        bail!("Cannot update server while it is running");
    }

    let mut manifest = ServerManifest::load(server_dir)
        .await
        .context("Failed to load server manifest")?;

    let Some(implementation) = get_server_implementation(&manifest.server_implementation) else {
        bail!(
            "Unknown server implementation '{}'",
            manifest.server_implementation
        );
    };

    let old_version = manifest.version.clone();
    let old_build = manifest.build.clone();

    let (latest_version, latest_build) = implementation
        .get_latest_version_build(match update_type {
            UpdateType::Stable => true,
            UpdateType::Latest => false,
        })
        .await
        .context("Failed to get latest version and build for server implementation")?;

    if old_version == latest_version.name && old_build == latest_build.name {
        return Ok(UpdateServerResult::NoUpdateNeeded);
    }

    manifest.version = latest_version.name.clone();
    manifest.build = latest_build.name.clone();

    let _server_jar_path = implementation
        .get_server_jar_path(server_dir, &manifest.version, &manifest.build)
        .await
        .context("Failed to prepare server jar for updated version")?;

    manifest
        .save(server_dir)
        .await
        .context("Failed to save updated server manifest")?;

    Ok(UpdateServerResult::Updated {
        old_version,
        old_build,
        new_version: manifest.version,
        new_build: manifest.build,
    })
}
