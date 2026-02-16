use std::path::Path;

use anyhow::{Context, bail};
use minecraftd_manifest::{JavaRuntime, ServerManifest};
use sha1::Digest;
use sha2::Sha256;

use crate::{
    server::implementations::ServerImplementation,
    util::{BoxedFuture, lazy_init_http_client::LazyInitHttpClient},
};

mod api;

const PROJECT_NAME: &str = "paper";

static CLIENT: LazyInitHttpClient = LazyInitHttpClient::new();

pub struct Paper;

impl ServerImplementation for Paper {
    fn name(&self) -> &str {
        "paper"
    }

    fn get_versions(&self) -> BoxedFuture<'static, anyhow::Result<Vec<String>>> {
        Box::pin(async move {
            let project = api::get_project(&CLIENT, PROJECT_NAME).await?;
            Ok(project.versions.into_iter().flat_map(|(_, v)| v).collect())
        })
    }

    fn get_builds<'a>(&self, version: &'a str) -> BoxedFuture<'a, anyhow::Result<Vec<String>>> {
        Box::pin(async move {
            let builds = api::get_builds(&CLIENT, PROJECT_NAME, version).await?;
            Ok(builds.into_iter().map(|b| b.id.to_string()).collect())
        })
    }

    fn create_server<'a>(
        &self,
        version: &'a str,
        build_str: &'a str,
        server_dir: &'a Path,
    ) -> BoxedFuture<'a, anyhow::Result<ServerManifest>> {
        Box::pin(async move {
            let build_num = build_str.parse::<u32>().context("Invalid build number")?;

            let build = api::get_build(&CLIENT, PROJECT_NAME, version, build_num)
                .await
                .context("Failed to get build info")?;

            let download = build
                .downloads
                .values()
                .next()
                .context("No downloads found")?;

            let mut expected_hash = [0u8; 32];
            hex::decode_to_slice(&download.checksums.sha256, &mut expected_hash)
                .context("Failed to decode SHA256 checksum")?;

            let file = CLIENT
                .get(&download.url)
                .send()
                .await
                .context("Failed to download server jar")?
                .bytes()
                .await
                .context("Failed to read server jar bytes")?;

            let downloaded_hash = Sha256::digest(&file);

            if downloaded_hash[..] != expected_hash {
                bail!("SHA256 checksum mismatch for downloaded server jar");
            }

            tokio::fs::write(server_dir.join("server.jar"), &file)
                .await
                .context("Failed to write server jar to disk")?;

            tokio::fs::write(server_dir.join("eula.txt"), "eula=true")
                .await
                .context("Failed to write eula.txt")?;

            let java_runtime =
                mojang_piston_api::minecraft::version_manifest::get_version_manifest()
                    .await
                    .context("Failed to fetch version manifest")?
                    .versions
                    .into_iter()
                    .find(|v| v.id == version.split('-').next().unwrap_or(""))
                    .context("Version not found in vanilla version manifest")?
                    .get()
                    .await
                    .context("Failed to fetch vanilla manifest")?
                    .java_version
                    .context("Java version info not found in vanilla manifest")?
                    .component;

            let default_manifest = ServerManifest::default(
                "paper",
                version,
                build_str,
                JavaRuntime::Mojang { name: java_runtime },
            );

            Ok(default_manifest)
        })
    }
}
