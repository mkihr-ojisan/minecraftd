use anyhow::{Context, bail};
use bytes::Bytes;
use minecraftd_manifest::JavaRuntime;
use sha1::{Digest, Sha1};

use crate::{
    server::implementations::ServerImplementation,
    util::{BoxedFuture, lazy_init_http_client::LazyInitHttpClient},
};

static CLIENT: LazyInitHttpClient = LazyInitHttpClient::new();

pub struct Vanilla;

impl ServerImplementation for Vanilla {
    fn name(&self) -> &'static str {
        "vanilla"
    }

    fn get_versions(&self) -> BoxedFuture<'static, anyhow::Result<Vec<String>>> {
        Box::pin(async move {
            let version_manifest = crate::util::cached_mojang_piston_api::get_version_manifest()
                .await
                .context("Failed to fetch version manifest")?;

            Ok(version_manifest
                .versions
                .into_iter()
                .map(|v| v.id)
                .collect::<Vec<_>>())
        })
    }

    fn get_builds<'a>(&self, version: &'a str) -> BoxedFuture<'a, anyhow::Result<Vec<String>>> {
        Box::pin(async move {
            // Vanilla does not have builds, so we just return the version as the only "build"
            Ok(vec![version.to_string()])
        })
    }

    fn default_java_runtime<'a>(
        &self,
        version: &'a str,
        _build: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<JavaRuntime>> {
        Box::pin(async move {
            let version_manifest = crate::util::cached_mojang_piston_api::get_version_manifest()
                .await
                .context("Failed to fetch version manifest")?;

            let version_info = version_manifest
                .versions
                .into_iter()
                .find(|v| v.id == version)
                .ok_or_else(|| anyhow::anyhow!("Version '{}' not found", version))?;

            let manifest = version_info
                .get()
                .await
                .context("Failed to fetch manifest")?;

            let component = manifest
                .java_version
                .context("Java version not specified in manifest")?
                .component;

            Ok(JavaRuntime::Mojang { name: component })
        })
    }

    fn download_server_jar<'a>(
        &self,
        version: &'a str,
        _build: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<Bytes>> {
        Box::pin(async move {
            let version_manifest = crate::util::cached_mojang_piston_api::get_version_manifest()
                .await
                .context("Failed to fetch version manifest")?;

            let version_info = version_manifest
                .versions
                .into_iter()
                .find(|v| v.id == version)
                .ok_or_else(|| anyhow::anyhow!("Version '{}' not found", version))?;

            let manifest = version_info
                .get()
                .await
                .context("Failed to fetch manifest")?;

            let Some(server) = manifest.downloads.server else {
                bail!("Server download not available for version '{}'", version);
            };

            let jar = CLIENT
                .get(&server.url)
                .send()
                .await
                .context("Failed to download server jar")?
                .bytes()
                .await
                .context("Failed to download server jar")?;

            let downloaded_hash = Sha1::digest(&jar);
            let expected_hash =
                hex::decode(&server.sha1).context("Failed to decode expected SHA1 hash")?;

            if downloaded_hash[..] != expected_hash[..] {
                bail!("Downloaded jar hash does not match expected hash");
            }

            Ok(jar)
        })
    }
}
