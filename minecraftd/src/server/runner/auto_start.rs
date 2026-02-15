use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};

static AUTO_START_CONFIG: Mutex<Option<AutoStartConfig>> = Mutex::const_new(None);

#[derive(Debug, Default, Serialize, Deserialize)]
struct AutoStartConfig {
    directories: HashSet<PathBuf>,
}

fn config_path() -> anyhow::Result<PathBuf> {
    let mut config_dir = dirs::data_dir().context("Failed to get configuration directory")?;
    config_dir.push("minecraftd");
    config_dir.push("auto_start.json");
    Ok(config_dir)
}

async fn load() -> anyhow::Result<AutoStartConfig> {
    let config_path = config_path()?;
    let content = tokio::fs::read_to_string(config_path)
        .await
        .context("Failed to read auto-start configuration file")?;
    let config =
        serde_json::from_str(&content).context("Failed to parse auto-start configuration file")?;
    Ok(config)
}

async fn save() -> anyhow::Result<()> {
    let config_path = config_path()?;
    tokio::fs::create_dir_all(
        config_path
            .parent()
            .expect("Configuration path should have a parent directory"),
    )
    .await
    .context("Failed to create directories for auto-start configuration file")?;
    let content = serde_json::to_string(&*get_auto_start_config().await)
        .context("Failed to serialize auto-start configuration")?;
    std::fs::write(config_path, content)
        .context("Failed to write auto-start configuration file")?;
    Ok(())
}

async fn get_auto_start_config() -> MappedMutexGuard<'static, AutoStartConfig> {
    let mut config_lock = AUTO_START_CONFIG.lock().await;
    if config_lock.is_none() {
        let config = load().await.unwrap_or_default();
        *config_lock = Some(config);
    }
    MutexGuard::map(config_lock, |opt| opt.as_mut().unwrap())
}

pub async fn add_auto_start_directory(server_dir: &Path) -> anyhow::Result<()> {
    let mut config = get_auto_start_config().await;
    if config.directories.insert(server_dir.to_path_buf()) {
        drop(config);
        save().await?;
    }
    Ok(())
}

pub async fn remove_auto_start_directory(server_dir: &Path) -> anyhow::Result<()> {
    let mut config = get_auto_start_config().await;
    if config.directories.remove(server_dir) {
        drop(config);
        save().await?;
    }
    Ok(())
}

pub async fn get_auto_start_directories() -> HashSet<PathBuf> {
    let config = get_auto_start_config().await;
    config.directories.clone()
}
