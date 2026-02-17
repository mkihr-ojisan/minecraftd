use std::path::Path;

use anyhow::{Context, bail};
use minecraftd_manifest::{Connection, ExntensionEntry, ExtensionType, ServerManifest};

use crate::server::{
    extension_providers::{ExtensionInfo, ExtensionProvider, get_extension_provider},
    implementations::{Build, Version, get_server_implementation},
    java_runtime::JavaRuntimeExt,
};

mod extension_providers;
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

pub async fn get_server_versions(server_implementation: &str) -> anyhow::Result<Vec<Version>> {
    let Some(implementation) = get_server_implementation(server_implementation) else {
        bail!("Unknown server implementation '{}'", server_implementation);
    };

    implementation.get_versions().await
}

pub async fn get_server_builds(
    server_implementation: &str,
    version: &str,
) -> anyhow::Result<Vec<Build>> {
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

pub fn get_extension_providers() -> impl Iterator<Item = &'static str> {
    extension_providers::EXTENSION_PROVIDERS
        .iter()
        .map(|p| p.name())
}

pub async fn search_extensions(
    provider: &str,
    type_: ExtensionType,
    server_version: &str,
    query: &str,
    include_incompatible_versions: bool,
) -> anyhow::Result<Vec<extension_providers::ExtensionInfo>> {
    let Some(provider) = extension_providers::EXTENSION_PROVIDERS
        .iter()
        .find(|p| p.name() == provider)
    else {
        bail!("Unknown extensioin provider '{}'", provider);
    };

    provider
        .search_extension(type_, server_version, query, include_incompatible_versions)
        .await
}

pub async fn get_extension_versions(
    provider: &str,
    type_: ExtensionType,
    server_version: &str,
    extension_id: &str,
    include_incompatible_versions: bool,
) -> anyhow::Result<Vec<extension_providers::ExtensionVersionInfo>> {
    let Some(provider) = extension_providers::EXTENSION_PROVIDERS
        .iter()
        .find(|p| p.name() == provider)
    else {
        bail!("Unknown extension provider '{}'", provider);
    };

    provider
        .get_extension_versions(
            type_,
            server_version,
            extension_id,
            include_incompatible_versions,
        )
        .await
}

pub struct AddExtensionResult {
    pub added_extensions: Vec<ExtensionInfo>,
}

pub async fn add_extension(
    server_dir: &Path,
    provider: &str,
    type_: ExtensionType,
    extension_id: &str,
    extension_version_id: &str,
    allow_incompatible_versions: bool,
) -> anyhow::Result<AddExtensionResult> {
    if runner::is_server_running(server_dir).await? {
        bail!("Cannot add extension while server is running");
    }

    let mut manifest = ServerManifest::load(server_dir)
        .await
        .context("Failed to load server manifest")?;

    let provider = get_extension_provider(provider).context("Unknown extension provider")?;

    #[async_recursion::async_recursion]
    async fn do_add_extension(
        manifest: &mut ServerManifest,
        added_extensions: &mut Vec<ExtensionInfo>,
        provider: &dyn ExtensionProvider,
        type_: ExtensionType,
        extension_id: &str,
        extension_version_id: Option<&str>,
        allow_incompatible_versions: bool,
    ) -> anyhow::Result<()> {
        // check if extension is already added with the same version (if version is not specified, just check if extension is already added)
        if manifest.extensions.iter().any(|m| {
            m.provider == provider.name()
                && m.id == extension_id
                && extension_version_id.is_none_or(|v| m.version_id == v)
        }) {
            return Ok(());
        }

        // remove any existing entry for the extension with a different version
        manifest
            .extensions
            .retain(|m| !(m.provider == provider.name() && m.id == extension_id));

        let extension_info = provider
            .get_extension_info(type_, extension_id)
            .await
            .context("Failed to get extension info")?;

        let version_info = if let Some(extension_version_id) = extension_version_id {
            provider
                .get_extension_version_info(type_, extension_id, extension_version_id)
                .await
                .context("Failed to get extension version info")?
        } else {
            let versions = provider
                .get_extension_versions(
                    type_,
                    &manifest.version,
                    extension_id,
                    allow_incompatible_versions,
                )
                .await
                .context("Failed to get extension versions")?;

            // latest stable version or latest version if no stable version is available
            let index = versions.iter().position(|v| v.is_stable).unwrap_or(0);

            versions
                .into_iter()
                .nth(index)
                .context("No versions found for extension")?
        };

        provider
            .get_extension_jar_path(type_, extension_id, &version_info.id)
            .await
            .context("Failed to prepare extension jar")?;

        manifest.extensions.push(ExntensionEntry {
            name: extension_info.name.clone(),
            type_,
            provider: provider.name().to_string(),
            id: extension_id.to_string(),
            version_id: version_info.id.clone(),
        });

        added_extensions.push(extension_info);

        for dependency in version_info.dependencies {
            do_add_extension(
                manifest,
                added_extensions,
                provider,
                type_,
                &dependency.extension_id,
                dependency.extension_version_id.as_deref(),
                allow_incompatible_versions,
            )
            .await
            .context(format!(
                "Failed to add dependency extension '{}' for extension '{}'",
                dependency.extension_id, extension_id
            ))?;
        }

        Ok(())
    }

    let mut added_extensions = Vec::new();

    do_add_extension(
        &mut manifest,
        &mut added_extensions,
        provider,
        type_,
        extension_id,
        Some(extension_version_id),
        allow_incompatible_versions,
    )
    .await?;

    manifest
        .save(server_dir)
        .await
        .context("Failed to save updated server manifest")?;

    Ok(AddExtensionResult { added_extensions })
}
