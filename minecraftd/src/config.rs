use std::sync::OnceLock;

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
}

impl Config {
    async fn load() -> anyhow::Result<Self> {
        let config_str = tokio::fs::read_to_string("config.toml").await?;
        let config: Self = toml::from_str(&config_str)?;
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
