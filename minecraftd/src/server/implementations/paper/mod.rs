use anyhow::{Context, bail};
use bytes::Bytes;
use minecraftd_manifest::JavaRuntime;
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
    fn name(&self) -> &'static str {
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

    fn default_java_runtime<'a>(
        &self,
        version: &'a str,
        _build: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<JavaRuntime>> {
        Box::pin(async move {
            let minecraft_version = version
                .split('-')
                .next()
                .context("Invalid version format")?;

            let component = crate::util::cached_mojang_piston_api::get_version_manifest()
                .await
                .context("Failed to fetch version manifest")?
                .versions
                .into_iter()
                .find(|v| v.id == minecraft_version)
                .context("Version not found in vanilla version manifest")?
                .get()
                .await
                .context("Failed to fetch vanilla manifest")?
                .java_version
                .context("Java version info not found in vanilla manifest")?
                .component;

            Ok(JavaRuntime::Mojang { name: component })
        })
    }

    fn download_server_jar<'a>(
        &self,
        version: &'a str,
        build_str: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<Bytes>> {
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

            Ok(file)
        })
    }
}
