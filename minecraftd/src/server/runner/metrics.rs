use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use anyhow::Context;
use tokio::{task::JoinHandle, time::MissedTickBehavior};
use tsink::{DataPoint, Label, QueryOptions, Row, Storage, StorageBuilder};

use crate::server::runner::RUNNER;

const METRICS_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);

static STORAGE: OnceLock<Arc<dyn Storage>> = OnceLock::new();
static METRICS_COLLECTOR_TASK: OnceLock<JoinHandle<()>> = OnceLock::new();

pub async fn init_metrics() -> anyhow::Result<()> {
    tokio::task::spawn_blocking(|| -> anyhow::Result<()> {
        let storage = StorageBuilder::new()
            .with_data_path(metrics_storage_path()?)
            .build()
            .context("Failed to initialize metrics storage")?;
        STORAGE
            .set(storage)
            .ok()
            .expect("init_metrics called multiple times");

        Ok(())
    })
    .await
    .unwrap()?;

    METRICS_COLLECTOR_TASK
        .set(tokio::spawn(async move {
            let mut interval = tokio::time::interval(METRICS_COLLECTION_INTERVAL);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                collect_metrics().await;
            }
        }))
        .expect("init_metrics called multiple times");

    Ok(())
}

pub async fn shutdown_metrics() {
    METRICS_COLLECTOR_TASK
        .get()
        .expect("Metrics collector task not initialized")
        .abort();

    tokio::task::spawn_blocking(|| {
        if let Err(err) = STORAGE
            .get()
            .expect("Metrics storage not initialized")
            .close()
        {
            error!("Failed to close metrics storage: {:?}", err);
        }
    })
    .await
    .unwrap();
}

fn metrics_storage_path() -> anyhow::Result<std::path::PathBuf> {
    let mut data_dir = dirs::data_dir().context("Failed to determine data directory")?;
    data_dir.push("minecraftd");
    data_dir.push("metrics");
    Ok(data_dir)
}

async fn collect_metrics() {
    let runner = RUNNER.lock().await;

    let mut metrics = Vec::new();

    for server in runner.running_servers.iter() {
        let Some(bridge) = server.bridge.get() else {
            continue;
        };

        let mut bridge = bridge.lock().await;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let server_id_str = server.manifest.id.to_string();

        match bridge.get_server_metrics().await {
            Ok(server_metrics) => {
                metrics.push(Row::with_labels(
                    "player_count",
                    vec![Label::new("server_id", &server_id_str)],
                    DataPoint::new(timestamp, server_metrics.player_count as f64),
                ));
            }
            Err(e) => error!(
                "Failed to collect metrics for server {}: {:?}",
                server.manifest.id, e
            ),
        }
    }

    tokio::task::spawn_blocking(move || {
        if let Err(err) = STORAGE
            .get()
            .expect("Metrics storage not initialized")
            .insert_rows(&metrics)
        {
            error!("Failed to insert metrics: {:?}", err);
        }
    })
    .await
    .unwrap();
}

pub async fn query_metrics(
    metric: impl Into<String>,
    opts: QueryOptions,
) -> anyhow::Result<Vec<DataPoint>> {
    let storage = STORAGE
        .get()
        .expect("Metrics storage not initialized")
        .clone();

    let metric = metric.into();

    tokio::task::spawn_blocking(move || {
        let rows = storage.select_with_options(&metric, opts)?;
        Ok(rows)
    })
    .await
    .unwrap()
}
