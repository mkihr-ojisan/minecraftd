use crate::{
    config::{WebhookConfig, WebhookType, get_config},
    util::lazy_init_http_client::LazyInitHttpClient,
};

static CLIENT: LazyInitHttpClient = LazyInitHttpClient::new();

#[derive(Debug, Clone)]
pub struct Alert {
    pub severity: Severity,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

pub async fn send_alert(alert_type: &str, alert: impl FnOnce() -> Alert) {
    let config = get_config();

    if config.alert.webhooks.is_empty() || config.alert.disabled_alert_types.contains(alert_type) {
        return;
    }

    let alert = alert();

    for webhook in &config.alert.webhooks {
        match webhook.type_ {
            WebhookType::Discord => {
                send_discord_webhook(webhook, alert_type, &alert).await;
            }
        }
    }
}

async fn send_discord_webhook(webhook: &WebhookConfig, alert_type: &str, alert: &Alert) {
    let payload = serde_json::json!({
        "username": env!("CARGO_PKG_NAME"),
        "embeds": [{
            "title": match alert.severity {
                Severity::Info => format!("ℹ️ {}", alert.title),
                Severity::Warning => format!("⚠️ {}", alert.title),
                Severity::Error => format!("❌ {}", alert.title),
            },
            "description": format!("{}\n\n-# type: {}", alert.message, alert_type),
            "color": match alert.severity {
                Severity::Info => 0x00FF00, // Green
                Severity::Warning => 0xFFFF00, // Yellow
                Severity::Error => 0xFF0000, // Red
            },
        }],
    });

    if let Err(e) = CLIENT.post(&webhook.url).json(&payload).send().await {
        error!("Failed to send alert to webhook '{}': {e:?}", webhook.name);
    }
}
