use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use bytes::Bytes;
use minecraftd_manifest::JavaRuntime;
use tokio::sync::Mutex;

use crate::util::BoxedFuture;

pub mod paper;
pub mod vanilla;

static SERVER_JAR_CACHE_LOCK: Mutex<()> = Mutex::const_new(());

pub trait ServerImplementation: Send + Sync {
    fn name(&self) -> &'static str;
    /// Ordered from newest to oldest
    fn get_versions<'a>(&'a self) -> BoxedFuture<'a, anyhow::Result<Vec<Version>>>;
    /// Ordered from newest to oldest
    fn get_builds<'a>(&'a self, version: &'a str) -> BoxedFuture<'a, anyhow::Result<Vec<Build>>>;
    fn default_java_runtime<'a>(
        &'a self,
        version: &'a str,
        build: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<JavaRuntime>>;
    fn download_server_jar<'a>(
        &'a self,
        version: &'a str,
        build: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<Bytes>>;

    fn get_server_jar_path<'a>(
        &'a self,
        _server_dir: &'a Path,
        version: &'a str,
        build: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<PathBuf>> {
        Box::pin(async move {
            let _lock = SERVER_JAR_CACHE_LOCK.lock().await;

            let mut cache_path = dirs::data_dir()
                .context("Could not determine data directory for caching server jars")?;
            cache_path.push("minecraftd");
            cache_path.push("versions");
            cache_path.push(self.name());
            cache_path.push(version);
            cache_path.push(build);
            cache_path.push("server.jar");

            if cache_path.exists() {
                return Ok(cache_path);
            }

            let jar_bytes = self.download_server_jar(version, build).await?;

            let parent_dir = cache_path.parent().unwrap();
            tokio::fs::create_dir_all(parent_dir)
                .await
                .with_context(|| {
                    format!(
                        "Failed to create cache directory for server jars at '{}'",
                        parent_dir.display()
                    )
                })?;
            tokio::fs::write(&cache_path, jar_bytes)
                .await
                .with_context(|| {
                    format!(
                        "Failed to write server jar to cache at '{}'",
                        cache_path.display()
                    )
                })?;

            Ok(cache_path)
        })
    }

    fn get_latest_version_build<'a>(
        &'a self,
        stable: bool,
    ) -> BoxedFuture<'a, anyhow::Result<(Version, Build)>> {
        Box::pin(async move {
            let versions = self.get_versions().await?;

            for version in versions {
                if stable && !version.is_stable {
                    continue;
                }

                let builds = self.get_builds(&version.name).await?;
                if let Some(build) = builds.into_iter().find(|build| !stable || build.is_stable) {
                    return Ok((version, build));
                }
            }

            bail!(
                "Could not find any versions or builds for server implementation '{}'",
                self.name()
            )
        })
    }

    fn is_newer_version_available<'a>(
        &'a self,
        current_version: &'a str,
        current_build: &'a str,
        stable: bool,
    ) -> BoxedFuture<'a, anyhow::Result<Option<(Version, Build)>>> {
        Box::pin(async move {
            let versions = self.get_versions().await?;
            for version in versions {
                if stable && !version.is_stable && version.name != current_version {
                    continue;
                }

                let builds = self.get_builds(&version.name).await?;
                for build in builds {
                    if version.name == current_version && build.name == current_build {
                        return Ok(None);
                    }

                    if !stable || (version.is_stable && build.is_stable) {
                        return Ok(Some((version, build)));
                    }
                }
            }

            Ok(None)
        })
    }
}

#[derive(Debug, Clone)]
pub struct Version {
    pub name: String,
    pub is_stable: bool,
}

#[derive(Debug, Clone)]
pub struct Build {
    pub name: String,
    pub is_stable: bool,
}

pub const SERVER_IMPLEMENTATIONS: &[&dyn ServerImplementation] =
    &[&vanilla::Vanilla, &paper::Paper];

pub fn get_server_implementation(name: &str) -> Option<&'static dyn ServerImplementation> {
    SERVER_IMPLEMENTATIONS
        .iter()
        .find(|impl_| impl_.name() == name)
        .copied()
}
