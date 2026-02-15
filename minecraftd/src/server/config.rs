use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::server::java_runtime::JavaRuntime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Connection {
    #[default]
    Direct,
    Proxy {
        hostname: String,
    },
}

impl ServerConfig {
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

    pub fn config_path(server_dir: &Path) -> std::path::PathBuf {
        server_dir.join("minecraftd.yaml")
    }

    pub async fn load(server_dir: &Path) -> anyhow::Result<Self> {
        let config_path = Self::config_path(server_dir);
        let config_data = tokio::fs::read_to_string(&config_path)
            .await
            .with_context(|| {
                format!("Failed to read config file at '{}'", config_path.display())
            })?;
        let config: ServerConfig = serde_yml::from_str(&config_data).with_context(|| {
            format!("Failed to parse config file at '{}'", config_path.display())
        })?;
        Ok(config)
    }

    pub async fn save(&self, server_dir: &Path) -> anyhow::Result<()> {
        let config_path = Self::config_path(server_dir);
        let config_data =
            serde_yml::to_string(self).context("Failed to serialize server config to YAML")?;
        tokio::fs::write(&config_path, config_data)
            .await
            .with_context(|| {
                format!("Failed to write config file at '{}'", config_path.display())
            })?;
        Ok(())
    }
}
