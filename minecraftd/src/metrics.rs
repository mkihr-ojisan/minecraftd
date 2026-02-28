use std::{
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

use crate::util::BoxedFuture;

const METRICS_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);
const METRICS_STORAGE_PARTITION_DURATION: Duration = Duration::from_secs(3600 * 24);
const METRICS_STORAGE_RETENTION: Duration = Duration::from_secs(3600 * 24 * 30);

static METRICS_MANAGER: OnceLock<Mutex<MetricsManager>> = OnceLock::new();

struct MetricsManager {
    storage: Arc<dyn Storage>,
    metrics_collectors: Vec<Box<dyn MetricsCollector>>,
}

pub trait MetricsCollector: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn collect<'a>(
        &'a mut self,
        ctx: &'a mut MetricsCollectorContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>>;
}

#[derive(Default)]
pub struct MetricsCollectorContext {
    rows: Vec<Row>,
}

impl MetricsCollectorContext {
    pub fn push_metric(&mut self, server_id: Uuid, metric: &str, timestamp: SystemTime, data: f64) {
        let row = Row::with_labels(
            metric,
            vec![Label::new("server_id", server_id.to_string())],
            DataPoint::new(
                timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                data,
            ),
        );
        self.rows.push(row);
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
            .with_partition_duration(METRICS_STORAGE_PARTITION_DURATION)
            .with_retention(METRICS_STORAGE_RETENTION)
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
        }))
        .ok()
        .expect("init_metrics called multiple times");

    tokio::spawn(async {
        let mut interval = tokio::time::interval(METRICS_COLLECTION_INTERVAL);
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

    if let Err(err) =
        tokio::task::spawn_blocking(move || manager.storage.insert_rows(&context.rows))
            .await
            .unwrap()
    {
        error!("Failed to insert metrics into storage: {:?}", err);
    }
}

pub async fn register_metrics_collector(collector: impl MetricsCollector) {
    let mut manager = get_metrics_manager().await;
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
