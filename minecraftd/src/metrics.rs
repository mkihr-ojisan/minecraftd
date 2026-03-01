use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime},
    vec,
};

use anyhow::Context;
use tokio::{
    sync::{Mutex, MutexGuard},
    time::MissedTickBehavior,
};
use tsink::{
    Aggregation, DataPoint, DownsampleOptions, Label, QueryOptions, Row, Storage, StorageBuilder,
};
use uuid::Uuid;

use crate::{
    alert::{self, Alert, Severity},
    config::get_config,
    runner,
    util::BoxedFuture,
};

static METRICS_MANAGER: OnceLock<Mutex<MetricsManager>> = OnceLock::new();

struct MetricsManager {
    storage: Arc<dyn Storage>,
    metrics_collectors: Vec<Box<dyn MetricsCollector>>,
    alert_rules: HashMap<&'static str, AlertRuleEntry>, // key: metric name
}

struct AlertRuleEntry {
    rule: &'static AlertRule,
    condition_met_since: Option<SystemTime>,
    alert_sent: bool,
}

pub trait MetricsCollector: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn collect<'a>(
        &'a mut self,
        ctx: &'a mut MetricsCollectorContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>>;
    fn alert_rules(&self) -> &'static [AlertRule] {
        &[]
    }
}

#[derive(Default)]
pub struct MetricsCollectorContext {
    metrics: Vec<MetricEntry>,
}

struct MetricEntry {
    server_id: Uuid,
    metric: String,
    timestamp: SystemTime,
    value: f64,
}

pub struct AlertRule {
    pub metric: &'static str,
    pub condition: AlertCondition,
    pub duration: Duration,
    pub alert_type: &'static str,
    pub alert_severity: Severity,
    pub alert_title: &'static str,
    pub alert_message: fn(&MetricAlertContext) -> String,
}

pub enum AlertCondition {
    GreaterThan { threshold: f64 },
    LessThan { threshold: f64 },
}

pub struct MetricAlertContext {
    pub server_dir: PathBuf,
    pub value: f64,
}

impl MetricsCollectorContext {
    pub fn push_metric(&mut self, server_id: Uuid, metric: &str, timestamp: SystemTime, data: f64) {
        self.metrics.push(MetricEntry {
            server_id,
            metric: metric.to_string(),
            timestamp,
            value: data,
        });
    }
}

fn metrics_storage_path() -> anyhow::Result<PathBuf> {
    let mut path = dirs::data_dir().context("Failed to get data directory")?;
    path.push("minecraftd");
    path.push("metrics");
    Ok(path)
}

async fn get_metrics_manager() -> MutexGuard<'static, MetricsManager> {
    METRICS_MANAGER
        .get()
        .expect("Metrics manager not initialized")
        .lock()
        .await
}

pub async fn init() -> anyhow::Result<()> {
    let storage = tokio::task::spawn_blocking(|| {
        StorageBuilder::new()
            .with_data_path(metrics_storage_path()?)
            .with_partition_duration(get_config().metrics.storage_partition_duration)
            .with_retention(get_config().metrics.storage_retention)
            .with_timestamp_precision(tsink::TimestampPrecision::Seconds)
            .build()
            .context("Failed to initialize metrics storage")
    })
    .await
    .unwrap()?;

    METRICS_MANAGER
        .set(Mutex::new(MetricsManager {
            storage,
            metrics_collectors: Vec::new(),
            alert_rules: HashMap::new(),
        }))
        .ok()
        .expect("init_metrics called multiple times");

    tokio::spawn(async {
        let mut interval = tokio::time::interval(get_config().metrics.collection_interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            collect_metrics().await;
        }
    });

    Ok(())
}

async fn collect_metrics() {
    let mut manager = get_metrics_manager().await;
    let mut context = MetricsCollectorContext::default();

    for collector in &mut manager.metrics_collectors {
        if let Err(err) = collector.collect(&mut context).await {
            error!(
                "Failed to collect metrics from collector '{}': {err:?}",
                collector.name(),
            );
        }
    }

    for metric in &context.metrics {
        if let Some(alert_rule_entry) = manager.alert_rules.get_mut(&*metric.metric) {
            let condition_met = match alert_rule_entry.rule.condition {
                AlertCondition::GreaterThan { threshold } => metric.value > threshold,
                AlertCondition::LessThan { threshold } => metric.value < threshold,
            };

            if condition_met {
                if let Some(since) = alert_rule_entry.condition_met_since
                    && let Ok(elapsed) = metric.timestamp.duration_since(since)
                {
                    if elapsed >= alert_rule_entry.rule.duration && !alert_rule_entry.alert_sent {
                        // Trigger alert
                        let Some(server_dir) = runner::get_server_dir(metric.server_id).await
                        else {
                            continue; // server stopped
                        };
                        let alert_context = MetricAlertContext {
                            server_dir,
                            value: metric.value,
                        };
                        let message = (alert_rule_entry.rule.alert_message)(&alert_context);

                        alert::send_alert(alert_rule_entry.rule.alert_type, || Alert {
                            title: alert_rule_entry.rule.alert_title.to_string(),
                            message,
                            severity: alert_rule_entry.rule.alert_severity,
                        })
                        .await;

                        alert_rule_entry.alert_sent = true;
                    }
                } else {
                    alert_rule_entry.condition_met_since = Some(metric.timestamp);
                    alert_rule_entry.alert_sent = false;
                }
            } else {
                alert_rule_entry.condition_met_since = None;
                alert_rule_entry.alert_sent = false;
            }
        }
    }

    let rows = context
        .metrics
        .into_iter()
        .map(|entry| {
            Row::with_labels(
                entry.metric,
                vec![Label::new("server_id", entry.server_id.to_string())],
                DataPoint {
                    value: entry.value,
                    timestamp: entry
                        .timestamp
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                },
            )
        })
        .collect::<Vec<_>>();

    if let Err(err) = tokio::task::spawn_blocking(move || manager.storage.insert_rows(&rows))
        .await
        .unwrap()
    {
        error!("Failed to insert metrics into storage: {:?}", err);
    }
}

pub async fn register_metrics_collector(collector: impl MetricsCollector) {
    let mut manager = get_metrics_manager().await;
    for alert_rule in collector.alert_rules() {
        manager.alert_rules.insert(
            alert_rule.metric,
            AlertRuleEntry {
                rule: alert_rule,
                condition_met_since: None,
                alert_sent: false,
            },
        );
    }
    manager.metrics_collectors.push(Box::new(collector));
}

pub async fn shutdown() {
    let manager = get_metrics_manager().await;
    if let Err(err) = tokio::task::spawn_blocking(move || manager.storage.close())
        .await
        .unwrap()
    {
        error!("Failed to close metrics storage: {:?}", err);
    }
}

pub struct MetricsQuery {
    pub server_id: Uuid,
    pub metric: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub aggregation: Aggregation,
    pub downsample_interval: Option<i64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn query(query: MetricsQuery) -> anyhow::Result<Vec<DataPoint>> {
    let manager = get_metrics_manager().await;

    tokio::task::spawn_blocking(move || {
        let rows = manager.storage.select_with_options(
            &query.metric,
            QueryOptions {
                labels: vec![Label::new("server_id", query.server_id.to_string())],
                start: query.start_timestamp,
                end: query.end_timestamp,
                aggregation: query.aggregation,
                downsample: query
                    .downsample_interval
                    .map(|interval| DownsampleOptions { interval }),
                limit: query.limit,
                offset: query.offset.unwrap_or(0),
            },
        )?;
        Ok(rows)
    })
    .await
    .unwrap()
}
