use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerManifest {
    pub name: String,
    pub server_implementation: String,
    pub version: String,
    pub build: String,
    #[serde(
        serialize_with = "serialize_command",
        deserialize_with = "deserialize_command"
    )]
    pub command: Vec<OsString>,
    pub java_runtime: JavaRuntime,
    #[serde(default)]
    pub restart_on_failure: bool,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default)]
    pub connection: Connection,
    #[serde(default)]
    pub extensions: Vec<ExtensionEntry>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ExtensionType,
    pub provider: String,
    pub id: String,
    pub version_id: String,
    #[serde(default)]
    pub auto_update: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionType {
    Mod,
    Plugin,
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
                OsString::from("${java}"),
                OsString::from("-Xmx4G"),
                OsString::from("-jar"),
                OsString::from("${server_jar}"),
                OsString::from("nogui"),
            ],
            java_runtime,
            restart_on_failure: true,
            auto_start: true,
            auto_update: false,
            connection: Connection::Direct,
            extensions: Vec::new(),
        }
    }

    pub fn manifest_path(server_dir: &Path) -> PathBuf {
        server_dir.join("minecraftd.yaml")
    }

    pub async fn load(server_dir: &Path) -> Result<Self, Error> {
        let manifest_path = Self::manifest_path(server_dir);
        let manifest_data = tokio::fs::read_to_string(&manifest_path).await?;
        let manifest: ServerManifest = serde_yml::from_str(&manifest_data)?;
        Ok(manifest)
    }

    pub async fn save(&self, server_dir: &Path) -> Result<(), Error> {
        let manifest_path = Self::manifest_path(server_dir);
        let manifest_data = serde_yml::to_string(self)?;
        tokio::fs::write(&manifest_path, manifest_data).await?;
        Ok(())
    }
}

fn serialize_command<S>(command: &Vec<OsString>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut command_strs = Vec::<String>::new();

    for arg in command {
        match arg.to_str() {
            Some(s) => command_strs.push(s.to_string()),
            None => {
                return Err(serde::ser::Error::custom(format!(
                    "Command argument '{arg:?}' is not valid UTF-8 and cannot be serialized"
                )));
            }
        }
    }

    command_strs.serialize(serializer)
}

fn deserialize_command<'de, D>(deserializer: D) -> Result<Vec<OsString>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let command_strs: Vec<String> = Vec::deserialize(deserializer)?;
    Ok(command_strs.into_iter().map(OsString::from).collect())
}
