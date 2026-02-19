use std::{collections::HashSet, path::PathBuf, sync::OnceLock};

use anyhow::Context;
use serde::Deserialize;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub async fn init_config() -> anyhow::Result<()> {
    let config = match Config::load().await {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("Failed to load config. Using default values. Error: {e:?}");
            Config::default()
        }
    };
    CONFIG.set(config).expect("Config already initialized");
    Ok(())
}

pub fn get_config() -> &'static Config {
    CONFIG.get().expect("Config not initialized")
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub port: PortConfig,
    #[serde(default)]
    pub proxy_server: ProxyServerConfig,
    #[serde(default)]
    pub alert: AlertConfig,
}

impl Config {
    fn config_path() -> anyhow::Result<PathBuf> {
        let mut path = dirs::config_dir().context("Failed to get config directory")?;
        path.push(env!("CARGO_PKG_NAME"));
        path.push("config.yaml");
        Ok(path)
    }

    async fn load() -> anyhow::Result<Self> {
        let config_str = tokio::fs::read_to_string(Self::config_path()?)
            .await
            .context("Failed to read config file")?;
        let config: Self = serde_yml::from_str(&config_str)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
pub struct PortConfig {
    #[serde(default = "default_port_min")]
    pub min: u16,
    #[serde(default = "default_port_max")]
    pub max: u16,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            min: default_port_min(),
            max: default_port_max(),
        }
    }
}

fn default_port_min() -> u16 {
    30001
}

fn default_port_max() -> u16 {
    30100
}

#[derive(Debug, Deserialize)]
pub struct ProxyServerConfig {
    #[serde(default = "default_proxy_server_bind_address")]
    pub bind_address: String,
}

impl Default for ProxyServerConfig {
    fn default() -> Self {
        Self {
            bind_address: default_proxy_server_bind_address(),
        }
    }
}

fn default_proxy_server_bind_address() -> String {
    "0.0.0.0:25565".to_string()
}

#[derive(Debug, Default, Deserialize)]
pub struct AlertConfig {
    #[serde(default)]
    pub webhooks: Vec<WebhookConfig>,
    #[serde(default)]
    pub disabled_alert_types: HashSet<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: WebhookType,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookType {
    Discord,
}
