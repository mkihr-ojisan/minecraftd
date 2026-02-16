use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::http_client::http_client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: VersionType,
    pub url: String,
    pub time: DateTime<FixedOffset>,
    pub release_time: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

pub async fn get_version_manifest() -> Result<VersionManifest, reqwest::Error> {
    http_client()
        .get("https://piston-meta.mojang.com/mc/game/version_manifest.json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

impl Version {
    pub async fn get(&self) -> Result<super::manifest::Manifest, reqwest::Error> {
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
    let manifest = get_version_manifest().await.unwrap();

    for v in &manifest.versions {
        eprintln!("Version: {}", v.id);
        v.get().await.unwrap();
    }
}
