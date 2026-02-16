use std::path::{Path, PathBuf};

use anyhow::Context;
use bytes::Bytes;
use minecraftd_manifest::JavaRuntime;
use tokio::sync::Mutex;

use crate::util::BoxedFuture;

pub mod paper;
pub mod vanilla;

static SERVER_JAR_CACHE_LOCK: Mutex<()> = Mutex::const_new(());

pub trait ServerImplementation: Send + Sync {
    fn name(&self) -> &'static str;
    fn get_versions<'a>(&'a self) -> BoxedFuture<'a, anyhow::Result<Vec<String>>>;
    fn get_builds<'a>(&'a self, version: &'a str) -> BoxedFuture<'a, anyhow::Result<Vec<String>>>;
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
            cache_path.push("server_jars");
            cache_path.push(self.name());
            cache_path.push(version);
            cache_path.push(build);
            cache_path.set_extension("jar");

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
}

pub const SERVER_IMPLEMENTATIONS: &[&dyn ServerImplementation] =
    &[&vanilla::Vanilla, &paper::Paper];

pub fn get_server_implementation(name: &str) -> Option<&'static dyn ServerImplementation> {
    SERVER_IMPLEMENTATIONS
        .iter()
        .find(|impl_| impl_.name() == name)
        .copied()
}
