use anyhow::Context;
use cached::proc_macro::cached;
use mojang_piston_api::minecraft::version_manifest::Version;

use std::time::Duration;

#[cached(result = true, time = 3600)]
pub async fn get_version_manifest()
-> anyhow::Result<mojang_piston_api::minecraft::version_manifest::VersionManifest> {
    let manifest = mojang_piston_api::minecraft::version_manifest::get_version_manifest()
        .await
        .context("Failed to fetch version manifest")?;

    Ok(manifest)
}

#[cached(
    result = true,
    time = 3600,
    key = "String",
    convert = r#"{ version.id.clone() }"#
)]
pub async fn get_manifest(
    version: &Version,
) -> anyhow::Result<mojang_piston_api::minecraft::manifest::Manifest> {
    let manifest = version
        .get()
        .await
        .context("Failed to fetch version manifest")?;

    Ok(manifest)
}
