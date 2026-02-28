use std::{collections::HashSet, path::PathBuf, sync::OnceLock, time::Duration};

use anyhow::Context;
use minecraft_protocol::text_component::{Color, Object, TextComponent};
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
    #[serde(default)]
    pub runner: RunnerConfig,
    #[serde(default)]
    pub auto_update: AutoUpdateConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
    #[serde(default)]
    pub messages: MessagesConfig,
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

#[derive(Debug, Deserialize)]
pub struct RunnerConfig {
    #[serde(
        default = "default_stop_timeout_secs",
        deserialize_with = "duration_str::deserialize_duration"
    )]
    pub stop_timeout: Duration,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            stop_timeout: default_stop_timeout_secs(),
        }
    }
}

fn default_stop_timeout_secs() -> Duration {
    Duration::from_secs(180)
}

#[derive(Debug, Deserialize)]
pub struct AutoUpdateConfig {
    #[serde(
        default = "default_update_check_interval_secs",
        deserialize_with = "duration_str::deserialize_duration"
    )]
    pub update_check_interval: Duration,
    #[serde(
        default = "default_wait_until_all_players_log_out_timeout",
        deserialize_with = "duration_str::deserialize_duration"
    )]
    pub wait_until_all_players_log_out_timeout: Duration,
    #[serde(
        default = "default_notify_players_before_restart_interval",
        deserialize_with = "duration_str::deserialize_duration"
    )]
    pub notify_players_before_restart_interval: Duration,
}

impl Default for AutoUpdateConfig {
    fn default() -> Self {
        Self {
            update_check_interval: default_update_check_interval_secs(),
            wait_until_all_players_log_out_timeout: default_wait_until_all_players_log_out_timeout(
            ),
            notify_players_before_restart_interval: default_notify_players_before_restart_interval(
            ),
        }
    }
}

fn default_update_check_interval_secs() -> Duration {
    Duration::from_hours(24)
}

fn default_wait_until_all_players_log_out_timeout() -> Duration {
    Duration::from_hours(1)
}

fn default_notify_players_before_restart_interval() -> Duration {
    Duration::from_mins(1)
}

#[derive(Debug, Deserialize)]
pub struct MetricsConfig {
    #[serde(
        default = "default_metrics_collection_interval",
        deserialize_with = "duration_str::deserialize_duration"
    )]
    pub collection_interval: Duration,
    #[serde(
        default = "default_metrics_storage_partition_duration",
        deserialize_with = "duration_str::deserialize_duration"
    )]
    pub storage_partition_duration: Duration,
    #[serde(
        default = "default_metrics_storage_retention",
        deserialize_with = "duration_str::deserialize_duration"
    )]
    pub storage_retention: Duration,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval: default_metrics_collection_interval(),
            storage_partition_duration: default_metrics_storage_partition_duration(),
            storage_retention: default_metrics_storage_retention(),
        }
    }
}

fn default_metrics_collection_interval() -> Duration {
    Duration::from_secs(1)
}

fn default_metrics_storage_partition_duration() -> Duration {
    Duration::from_secs(3600 * 24)
}

fn default_metrics_storage_retention() -> Duration {
    Duration::from_secs(3600 * 24 * 30)
}

#[derive(Debug, Deserialize)]
pub struct MessagesConfig {
    #[serde(default = "default_server_restarting_for_update_message")]
    pub server_restarting_for_update: TextComponent,
    #[serde(default = "default_server_not_found_message")]
    pub server_not_found: TextComponent,
    #[serde(default = "default_server_starting_message")]
    pub server_starting: TextComponent,
    #[serde(default = "default_server_stopping_message")]
    pub server_stopping: TextComponent,
    #[serde(default = "default_server_restarting_message")]
    pub server_restarting: TextComponent,
    #[serde(default = "default_server_restarting_kick_message")]
    pub server_restarting_kick: String,
}

impl Default for MessagesConfig {
    fn default() -> Self {
        Self {
            server_restarting_for_update: default_server_starting_message(),
            server_not_found: default_server_not_found_message(),
            server_starting: default_server_starting_message(),
            server_stopping: default_server_stopping_message(),
            server_restarting: default_server_restarting_message(),
            server_restarting_kick: default_server_restarting_kick_message(),
        }
    }
}

fn default_server_restarting_for_update_message() -> TextComponent {
    TextComponent::Object(Object {
        text: Some("".to_string()),
        extra: Some(vec![
            TextComponent::Object(Object {
                text: Some("=== SERVER RESTART ===".to_string()),
                color: Some(Color::Red),
                ..Default::default()
            }),
            TextComponent::String("\nThe server will restart in one minute to apply the updates.\nPlease log out to avoid interruptions.".to_string()),
        ]),
        ..Default::default()
    })
}

fn default_server_starting_message() -> TextComponent {
    TextComponent::String("The server is starting up, please try again later.".to_string())
}

fn default_server_not_found_message() -> TextComponent {
    TextComponent::String("The server is not running or does not exist.".to_string())
}

fn default_server_stopping_message() -> TextComponent {
    TextComponent::String("The server is stopping.".to_string())
}

fn default_server_restarting_message() -> TextComponent {
    TextComponent::String("The server is restarting, please try again later.".to_string())
}

fn default_server_restarting_kick_message() -> String {
    "The server is restarting, please try connecting again after a while.".to_string()
}
