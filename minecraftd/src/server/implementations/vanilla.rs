use anyhow::{Context, bail};
use sha1::{Digest, Sha1};

use crate::{
    server::{
        config::ServerConfig, implementations::ServerImplementation, java_runtime::JavaRuntime,
    },
    util::{BoxedFuture, lazy_init_http_client::LazyInitHttpClient},
};

static CLIENT: LazyInitHttpClient = LazyInitHttpClient::new();

pub struct Vanilla;

impl ServerImplementation for Vanilla {
    fn name(&self) -> &str {
        "vanilla"
    }

    fn get_versions(&self) -> BoxedFuture<'static, anyhow::Result<Vec<String>>> {
        Box::pin(async move {
            let version_manifest =
                mojang_piston_api::minecraft::version_manifest::get_version_manifest()
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

    fn create_server<'a>(
        &self,
        version: &'a str,
        _build: &'a str,
        server_dir: &'a std::path::Path,
    ) -> BoxedFuture<'a, anyhow::Result<ServerConfig>> {
        Box::pin(async move {
            let version_manifest =
                mojang_piston_api::minecraft::version_manifest::get_version_manifest()
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

            tokio::fs::write(server_dir.join("server.jar"), &jar)
                .await
                .context("Failed to write server jar to disk")?;

            tokio::fs::write(server_dir.join("eula.txt"), "eula=true")
                .await
                .context("Failed to write eula.txt")?;

            let default_config = ServerConfig::default(
                "vanilla",
                version,
                version,
                JavaRuntime::Mojang {
                    name: manifest
                        .java_version
                        .as_ref()
                        .map(|jv| &*jv.component)
                        .unwrap_or("java-runtime-delta")
                        .to_string(),
                },
            );

            Ok(default_config)
        })
    }
}
