use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use anyhow::Context;
use minecraftd_manifest::ExtensionType;

use crate::extension::providers::ExtensionProvider;

pub async fn get_or_download(
    provider: &dyn ExtensionProvider,
    type_: ExtensionType,
    extension_id: &str,
    extension_version_id: &str,
) -> anyhow::Result<PathBuf> {
    let path = extension_cache_path(provider.name(), type_, extension_id, extension_version_id)?;

    if path.exists() {
        return Ok(path);
    }

    let bytes = provider
        .download_extension_jar(type_, extension_id, extension_version_id)
        .await?;

    tokio::fs::create_dir_all(path.parent().unwrap())
        .await
        .context("Failed to create extension directory")?;
    tokio::fs::write(&path, bytes)
        .await
        .context("Failed to write extension jar to disk")?;

    Ok(path)
}

pub struct ExtensionSymlinkInfo {
    pub provider: OsString,
    pub ty: ExtensionType,
    pub id: OsString,
    pub version_id: OsString,
}

pub async fn get_extension_symlink_info(
    path: &Path,
) -> anyhow::Result<Option<ExtensionSymlinkInfo>> {
    let target = tokio::fs::read_link(&path).await?;

    let Ok(relative_target) = target.strip_prefix(extension_cache_root_dir()?) else {
        return Ok(None);
    };

    let components = relative_target.components().collect::<Vec<_>>();
    if components.len() != 5 {
        return Ok(None);
    }

    let provider = components[0].as_os_str().to_os_string();

    let type_ = match components[1].as_os_str().to_str() {
        Some("mods") => ExtensionType::Mod,
        Some("plugins") => ExtensionType::Plugin,
        _ => return Ok(None),
    };

    let id = components[2].as_os_str().to_os_string();
    let version_id = components[3].as_os_str().to_os_string();

    if components[4].as_os_str() != "extension.jar" {
        return Ok(None);
    }

    Ok(Some(ExtensionSymlinkInfo {
        provider,
        ty: type_,
        id,
        version_id,
    }))
}

fn extension_cache_root_dir() -> anyhow::Result<PathBuf> {
    let mut path = dirs::data_dir().context("Could not find data directory")?;
    path.push("minecraftd");
    path.push("extensions");
    Ok(path)
}

fn extension_cache_path(
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
