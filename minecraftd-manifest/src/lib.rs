use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerManifest {
    pub name: String,
    pub server_implementation: String,
    pub version: String,
    pub build: String,
    pub command: Vec<String>,
    pub java_runtime: JavaRuntime,
    #[serde(default)]
    pub restart_on_failure: bool,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub connection: Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JavaRuntime {
    Mojang { name: String },
    Custom { java_home: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Connection {
    #[default]
    Direct,
    Proxy {
        hostname: String,
    },
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse manifest file: {0}")]
    ParseError(#[from] serde_yml::Error),
}

impl ServerManifest {
    pub fn default(
        server_implementation: &str,
        version: &str,
        build: &str,
        java_runtime: JavaRuntime,
    ) -> Self {
        Self {
            name: String::new(),
            server_implementation: server_implementation.to_string(),
            version: version.to_string(),
            build: build.to_string(),
            command: vec![
                "${java}".to_string(),
                "-Xmx4G".to_string(),
                "-jar".to_string(),
                "server.jar".to_string(),
                "nogui".to_string(),
            ],
            java_runtime,
            restart_on_failure: true,
            auto_start: true,
            connection: Connection::Direct,
        }
    }

    pub fn manifest_path(server_dir: &Path) -> PathBuf {
        server_dir.join("minecraftd.yaml")
    }

    pub async fn load(server_dir: &Path) -> Result<Self, Error> {
        let manifest_path = Self::manifest_path(server_dir);
        let manifest_data = tokio::fs::read_to_string(&manifest_path).await?;
        let manigest: ServerManifest = serde_yml::from_str(&manifest_data)?;
        Ok(manigest)
    }

    pub async fn save(&self, server_dir: &Path) -> Result<(), Error> {
        let manifest_path = Self::manifest_path(server_dir);
        let manifest_data = serde_yml::to_string(self)?;
        tokio::fs::write(&manifest_path, manifest_data).await?;
        Ok(())
    }
}
