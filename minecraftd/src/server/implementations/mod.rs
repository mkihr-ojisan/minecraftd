use std::path::Path;

use minecraftd_manifest::ServerManifest;

use crate::util::BoxedFuture;

pub mod paper;
pub mod vanilla;

pub trait ServerImplementation: Send + Sync {
    fn name(&self) -> &str;
    fn get_versions(&self) -> BoxedFuture<'static, anyhow::Result<Vec<String>>>;
    fn get_builds<'a>(&self, version: &'a str) -> BoxedFuture<'a, anyhow::Result<Vec<String>>>;
    fn create_server<'a>(
        &self,
        version: &'a str,
        build: &'a str,
        server_dir: &'a Path,
    ) -> BoxedFuture<'a, anyhow::Result<ServerManifest>>;
}

pub const SERVER_IMPLEMENTATIONS: &[&dyn ServerImplementation] =
    &[&vanilla::Vanilla, &paper::Paper];

pub fn get_server_implementation(name: &str) -> Option<&'static dyn ServerImplementation> {
    SERVER_IMPLEMENTATIONS
        .iter()
        .find(|impl_| impl_.name() == name)
        .copied()
}
