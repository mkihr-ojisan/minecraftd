use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub files: HashMap<String, File>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum File {
    Directory,
    File {
        downloads: Downloads,
        executable: bool,
    },
    Link {
        target: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    pub lzma: Option<Download>,
    pub raw: Option<Download>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}
