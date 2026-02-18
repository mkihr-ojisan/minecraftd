use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::http_client::http_client;

pub mod manifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct All {
    pub gamecore: HashMap<String, Vec<Runtime>>,
    pub linux: HashMap<String, Vec<Runtime>>,
    pub linux_i386: HashMap<String, Vec<Runtime>>,
    pub mac_os: HashMap<String, Vec<Runtime>>,
    pub mac_os_arm64: HashMap<String, Vec<Runtime>>,
    pub windows_arm64: HashMap<String, Vec<Runtime>>,
    pub windows_x64: HashMap<String, Vec<Runtime>>,
    pub windows_x86: HashMap<String, Vec<Runtime>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runtime {
    pub availability: Availability,
    pub manifest: Manifest,
    pub version: Version,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Availability {
    pub group: i32,
    pub progress: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub released: DateTime<FixedOffset>,
}

pub async fn get_all_java_runtimes() -> reqwest::Result<All> {
    http_client()
        .get("https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

impl Manifest {
    pub async fn get(&self) -> reqwest::Result<manifest::Manifest> {
        http_client()
            .get(&self.url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

#[cfg(test)]
#[tokio::test]
async fn test() {
    let runtimes = get_all_java_runtimes().await.unwrap();

    for r in [
        &runtimes.gamecore,
        &runtimes.linux,
        &runtimes.linux_i386,
        &runtimes.mac_os,
        &runtimes.mac_os_arm64,
        &runtimes.windows_arm64,
        &runtimes.windows_x64,
        &runtimes.windows_x86,
    ]
    .iter()
    .flat_map(|r| r.values().flatten())
    {
        r.manifest.get().await.unwrap();
    }
}
