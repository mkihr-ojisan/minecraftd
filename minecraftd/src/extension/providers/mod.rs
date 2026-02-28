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

    fn is_newer_version_available<'a>(
        &'a self,
        type_: ExtensionType,
        server_version: &'a str,
        extension_id: &'a str,
        current_extension_version_id: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<Option<ExtensionVersionInfo>>> {
        Box::pin(async move {
            let versions = self
                .get_extension_versions(type_, server_version, extension_id, false)
                .await?;

            let Some(latest_stable_version) = versions
                .into_iter()
                .take_while(|version| version.id != current_extension_version_id)
                .find(|version| version.is_stable)
            else {
                return Ok(None);
            };

            Ok(Some(latest_stable_version))
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
    pub dependencies: Vec<ExtensionDependency>,
}

pub struct ExtensionDependency {
    pub extension_id: String,
    pub extension_version_id: Option<String>,
}

pub const EXTENSION_PROVIDERS: &[&dyn ExtensionProvider] = &[&modrinth::Modrinth];

pub fn get_extension_provider(name: &str) -> Option<&'static dyn ExtensionProvider> {
    EXTENSION_PROVIDERS
        .iter()
        .find(|provider| provider.name() == name)
        .copied()
}
