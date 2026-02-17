use std::path::PathBuf;

use anyhow::Context;
use bytes::Bytes;
use minecraftd_manifest::ExtensionType;

use crate::util::BoxedFuture;

pub mod modrinth;

pub trait ExtensionProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn search_extension<'a>(
        &'a self,
        type_: ExtensionType,
        server_version: &'a str,
        query: &'a str,
        include_incompatible_versions: bool,
    ) -> BoxedFuture<'a, anyhow::Result<Vec<ExtensionInfo>>>;
    fn get_extension_info<'a>(
        &'a self,
        type_: ExtensionType,
        extension_id: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<ExtensionInfo>>;
    fn get_extension_versions<'a>(
        &'a self,
        type_: ExtensionType,
        server_version: &'a str,
        extension_id: &'a str,
        include_incompatible_versions: bool,
    ) -> BoxedFuture<'a, anyhow::Result<Vec<ExtensionVersionInfo>>>;
    fn get_extension_version_info<'a>(
        &'a self,
        type_: ExtensionType,
        extension_id: &'a str,
        extension_version_id: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<ExtensionVersionInfo>>;
    fn download_extension_jar<'a>(
        &'a self,
        type_: ExtensionType,
        extension_id: &'a str,
        extension_version_id: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<Bytes>>;
    fn get_extension_info_by_url<'a>(
        &'a self,
        url: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<ExtensionInfo>>;

    fn get_extension_jar_path<'a>(
        &'a self,
        type_: ExtensionType,
        extension_id: &'a str,
        extension_version_id: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<PathBuf>> {
        Box::pin(async move {
            let path =
                extension_cache_path(self.name(), type_, extension_id, extension_version_id)?;

            if path.exists() {
                return Ok(path);
            }

            let bytes = self
                .download_extension_jar(type_, extension_id, extension_version_id)
                .await?;

            tokio::fs::create_dir_all(path.parent().unwrap())
                .await
                .context("Failed to create extension directory")?;
            tokio::fs::write(&path, bytes)
                .await
                .context("Failed to write extension jar to disk")?;

            Ok(path)
        })
    }
}

pub struct ExtensionInfo {
    pub id: String,
    pub type_: ExtensionType,
    pub name: String,
}

pub struct ExtensionVersionInfo {
    pub id: String,
    pub version: String,
    pub is_stable: bool,
    pub dependencies: Vec<ExntensionDependency>,
}

pub struct ExntensionDependency {
    pub extension_id: String,
    pub extension_version_id: Option<String>,
}

pub const EXTENSION_PROVIDERS: &[&dyn ExtensionProvider] = &[&modrinth::Modrinth];

pub fn extension_cache_root_dir() -> anyhow::Result<PathBuf> {
    let mut path = dirs::data_dir().context("Could not find data directory")?;
    path.push("minecraftd");
    path.push("extensions");
    Ok(path)
}

pub fn extension_cache_path(
    provider: &str,
    type_: ExtensionType,
    id: &str,
    version_id: &str,
) -> anyhow::Result<PathBuf> {
    let mut path = extension_cache_root_dir()?;
    path.push(provider);
    path.push(match type_ {
        ExtensionType::Mod => "mods",
        ExtensionType::Plugin => "plugins",
    });
    path.push(id);
    path.push(version_id);
    path.push("extension.jar");
    Ok(path)
}

pub fn get_extension_provider(name: &str) -> Option<&'static dyn ExtensionProvider> {
    EXTENSION_PROVIDERS
        .iter()
        .find(|provider| provider.name() == name)
        .copied()
}
