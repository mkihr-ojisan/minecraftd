use std::sync::LazyLock;

use anyhow::Context;
use minecraftd_manifest::ExtensionType;
use modrinth_api::{
    apis::configuration::Configuration,
    models::{version::VersionType, version_dependency::DependencyType},
};

use crate::{
    server::extension_providers::{
        ExntensionDependency, ExtensionInfo, ExtensionProvider, ExtensionVersionInfo,
    },
    util::BoxedFuture,
};

pub struct Modrinth;

static CONFIG: LazyLock<Configuration> = LazyLock::new(Configuration::new);

impl ExtensionProvider for Modrinth {
    fn name(&self) -> &'static str {
        "modrinth"
    }

    fn search_extension<'a>(
        &'a self,
        type_: ExtensionType,
        server_version: &'a str,
        query: &'a str,
        include_incompatible_versions: bool,
    ) -> BoxedFuture<'a, anyhow::Result<Vec<ExtensionInfo>>> {
        let facets = if include_incompatible_versions {
            None
        } else {
            let mut facets = vec![format!(r#"["versions:{server_version}"]"#)];

            match type_ {
                ExtensionType::Mod => facets.push(r#"["project_type:mod"]"#.to_string()),
                ExtensionType::Plugin => facets.push(r#"["project_type:plugin"]"#.to_string()),
            }

            Some(format!("[{}]", facets.join(",")))
        };

        Box::pin(async move {
            let search_result = modrinth_api::apis::projects_api::search_projects(
                &CONFIG,
                Some(query),
                facets.as_deref(),
                None,
                None,
                None,
            )
            .await?;

            Ok(search_result
                .hits
                .into_iter()
                .map(|hit| ExtensionInfo {
                    id: hit.project_id,
                    name: hit.title,
                })
                .collect::<Vec<_>>())
        })
    }

    fn get_extension_info<'a>(
        &'a self,
        _type: ExtensionType,
        extension_id: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<ExtensionInfo>> {
        Box::pin(async move {
            let project =
                modrinth_api::apis::projects_api::get_project(&CONFIG, extension_id).await?;

            Ok(ExtensionInfo {
                id: project.id,
                name: project.title,
            })
        })
    }

    fn get_extension_versions<'a>(
        &'a self,
        _type: ExtensionType,
        server_version: &'a str,
        extension_id: &'a str,
        include_incompatible_versions: bool,
    ) -> BoxedFuture<'a, anyhow::Result<Vec<ExtensionVersionInfo>>> {
        Box::pin(async move {
            let versions = modrinth_api::apis::versions_api::get_project_versions(
                &CONFIG,
                extension_id,
                None,
                if include_incompatible_versions {
                    None
                } else {
                    Some(format!(r#"["{server_version}"]"#))
                }
                .as_deref(),
                None,
                None,
            )
            .await?;

            Ok(versions
                .into_iter()
                .flat_map(|v| {
                    Some(ExtensionVersionInfo {
                        id: v.id,
                        version: v.version_number,
                        is_stable: v.version_type == VersionType::Release,
                        dependencies: v
                            .dependencies
                            .into_iter()
                            .flatten()
                            .flat_map(|d| {
                                if d.dependency_type == DependencyType::Required {
                                    Some(ExntensionDependency {
                                        extension_id: d.project_id??,
                                        extension_version_id: d.version_id?,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    })
                })
                .collect::<Vec<_>>())
        })
    }

    fn get_extension_version_info<'a>(
        &'a self,
        _type: ExtensionType,
        _extension_id: &'a str,
        extension_version_id: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<ExtensionVersionInfo>> {
        Box::pin(async move {
            let version =
                modrinth_api::apis::versions_api::get_version(&CONFIG, extension_version_id)
                    .await?;

            Ok(ExtensionVersionInfo {
                id: version.id,
                version: version.version_number,
                is_stable: version.version_type == VersionType::Release,
                dependencies: version
                    .dependencies
                    .into_iter()
                    .flatten()
                    .flat_map(|d| {
                        if d.dependency_type == DependencyType::Required {
                            Some(ExntensionDependency {
                                extension_id: d.project_id??,
                                extension_version_id: d.version_id?,
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
            })
        })
    }

    fn download_extension_jar<'a>(
        &'a self,
        _type: ExtensionType,
        _extension_id: &'a str,
        extension_version_id: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<bytes::Bytes>> {
        Box::pin(async move {
            let version =
                modrinth_api::apis::versions_api::get_version(&CONFIG, extension_version_id)
                    .await?;

            let file = version
                .files
                .into_iter()
                .next()
                .context("Version has no files")?;

            let response = CONFIG.client.get(&file.url).send().await?;
            let bytes = response.bytes().await?;

            Ok(bytes)
        })
    }
}
