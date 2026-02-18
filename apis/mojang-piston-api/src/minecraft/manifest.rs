use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use super::version_manifest::VersionType;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub arguments: Option<Arguments>,
    pub asset_index: AssetIndex,
    pub assets: String,
    pub compliance_level: Option<i32>,
    pub downloads: Downloads,
    pub id: String,
    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,
    pub logging: Option<Logging>,
    pub main_class: String,
    pub minimum_launcher_version: i32,
    pub release_time: DateTime<FixedOffset>,
    pub time: DateTime<FixedOffset>,
    #[serde(rename = "type")]
    pub type_: VersionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    ArgumentWithRules(ArgumentWithRules),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentWithRules {
    pub rules: Vec<Rule>,
    pub value: ArgumentWithRulesValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentWithRulesValue {
    String(String),
    MultipleString(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: RuleAction,
    pub features: Option<HashMap<Feature, bool>>,
    pub os: Option<Os>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    IsDemoUser,
    HasCustomResolution,
    HasQuickPlaysSupport,
    IsQuickPlaySingleplayer,
    IsQuickPlayMultiplayer,
    IsQuickPlayRealms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Os {
    pub name: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    pub client: Download,
    pub client_mappings: Option<Download>,
    pub server: Option<Download>,
    pub server_mappings: Option<Download>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub downloads: Option<LibraryDownloads>,
    pub name: String,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    pub classifiers: Option<HashMap<String, LibraryArtifact>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryArtifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingClient {
    pub argument: String,
    pub file: LoggingClientFile,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingClientFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}
