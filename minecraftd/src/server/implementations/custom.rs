use std::path::{Path, PathBuf};

use bytes::Bytes;
use minecraftd_manifest::JavaRuntime;

use crate::{
    server::implementations::{Build, ServerImplementation, Version},
    util::BoxedFuture,
};

pub struct Custom;

impl ServerImplementation for Custom {
    fn name(&self) -> &'static str {
        "custom"
    }

    fn get_versions<'a>(&'a self) -> BoxedFuture<'a, anyhow::Result<Vec<Version>>> {
        Box::pin(async move {
            Ok(vec![Version {
                name: "".to_string(),
                is_stable: true,
            }])
        })
    }

    fn get_builds<'a>(&'a self, _version: &'a str) -> BoxedFuture<'a, anyhow::Result<Vec<Build>>> {
        Box::pin(async move {
            Ok(vec![Build {
                name: "".to_string(),
                is_stable: true,
            }])
        })
    }

    fn default_java_runtime<'a>(
        &'a self,
        _version: &'a str,
        _build: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<minecraftd_manifest::JavaRuntime>> {
        Box::pin(async move {
            Ok(JavaRuntime::Mojang {
                name: "java-runtime-delta".to_string(),
            })
        })
    }

    fn download_server_jar<'a>(
        &'a self,
        _version: &'a str,
        _build: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<bytes::Bytes>> {
        Box::pin(async move { Ok(Bytes::new()) })
    }

    fn get_server_jar_path<'a>(
        &'a self,
        _server_dir: &'a Path,
        _version: &'a str,
        _build: &'a str,
    ) -> BoxedFuture<'a, anyhow::Result<std::path::PathBuf>> {
        Box::pin(async move { Ok(PathBuf::new()) })
    }

    fn get_latest_version_build<'a>(
        &'a self,
        stable: bool,
    ) -> BoxedFuture<'a, anyhow::Result<(Version, Build)>> {
        Box::pin(async move {
            Ok((
                Version {
                    name: "".to_string(),
                    is_stable: stable,
                },
                Build {
                    name: "".to_string(),
                    is_stable: stable,
                },
            ))
        })
    }

    fn is_newer_version_available<'a>(
        &'a self,
        _current_version: &'a str,
        _current_build: &'a str,
        _stable: bool,
    ) -> BoxedFuture<'a, anyhow::Result<Option<(Version, Build)>>> {
        Box::pin(async move { Ok(None) })
    }
}
