use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use anyhow::Context;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};
use tokio::{sync::MutexGuard, task::JoinHandle, time::MissedTickBehavior};
use tsink::{DataPoint, Label, QueryOptions, Row, Storage, StorageBuilder, TimestampPrecision};
use uuid::Uuid;

use crate::server::{
    proxy_server::{self, ProxyStats},
    runner::{RUNNER, Runner},
};

const METRICS_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);

static STORAGE: OnceLock<Arc<dyn Storage>> = OnceLock::new();
static METRICS_COLLECTOR_TASK: OnceLock<JoinHandle<()>> = OnceLock::new();

struct LastValue {
    proxy: (Instant, HashMap<Uuid, ProxyStats>),
    system: (Instant, sysinfo::System),
}

pub async fn init_metrics() -> anyhow::Result<()> {
    tokio::task::spawn_blocking(|| -> anyhow::Result<()> {
        let storage = StorageBuilder::new()
            .with_data_path(metrics_storage_path()?)
            .with_partition_duration(Duration::from_secs(3600))
            .with_retention(Duration::from_secs(3600 * 24 * 7))
            .with_timestamp_precision(TimestampPrecision::Seconds) // 7 days
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

            let mut last_values = LastValue {
                proxy: (Instant::now(), HashMap::new()),
                system: (Instant::now(), sysinfo::System::new()),
            };

            loop {
                interval.tick().await;
                collect_metrics(&mut last_values).await;
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

async fn collect_metrics(last_values: &mut LastValue) {
    let runner = RUNNER.lock().await;

    let mut metrics = Vec::new();

    collect_bridge_metrics(&mut metrics, &runner).await;
    collect_proxy_metrics(&mut metrics, last_values).await;
    collect_system_metrics(&mut metrics, &runner, last_values);

    if metrics.is_empty() {
        return;
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

async fn collect_bridge_metrics(metrics: &mut Vec<Row>, runner: &MutexGuard<'_, Runner>) {
    for server in runner.running_servers.iter() {
        let Some(bridge) = server.bridge.get() else {
            continue;
        };

        let mut bridge = bridge.lock().await;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let labels = vec![Label::new("server_id", server.manifest.id.to_string())];

        match bridge.get_server_metrics().await {
            Ok(server_metrics) => {
                if let Some(tps) = server_metrics.tps {
                    metrics.push(Row::with_labels(
                        "tps",
                        labels.clone(),
                        DataPoint::new(timestamp, tps),
                    ));
                }
                if let Some(mspt) = server_metrics.mspt {
                    metrics.push(Row::with_labels(
                        "mspt",
                        labels.clone(),
                        DataPoint::new(timestamp, mspt),
                    ));
                }
                if let Some(player_count) = server_metrics.player_count {
                    metrics.push(Row::with_labels(
                        "player_count",
                        labels.clone(),
                        DataPoint::new(timestamp, player_count as f64),
                    ));
                }
                if let Some(entity_count) = server_metrics.entity_count {
                    metrics.push(Row::with_labels(
                        "entity_count",
                        labels.clone(),
                        DataPoint::new(timestamp, entity_count as f64),
                    ));
                }
                if let Some(loaded_chunk_count) = server_metrics.loaded_chunk_count {
                    metrics.push(Row::with_labels(
                        "loaded_chunk_count",
                        labels.clone(),
                        DataPoint::new(timestamp, loaded_chunk_count as f64),
                    ));
                }
                if let Some(allocated_memory) = server_metrics.allocated_memory {
                    metrics.push(Row::with_labels(
                        "allocated_memory_bytes",
                        labels.clone(),
                        DataPoint::new(timestamp, allocated_memory as f64),
                    ));
                }
                if let Some(used_memory) = server_metrics.used_memory {
                    metrics.push(Row::with_labels(
                        "used_memory_bytes",
                        labels.clone(),
                        DataPoint::new(timestamp, used_memory as f64),
                    ));
                }
            }
            Err(e) => error!(
                "Failed to collect metrics for server {}: {:?}",
                server.manifest.id, e
            ),
        }
    }
}

async fn collect_proxy_metrics(metrics: &mut Vec<Row>, last_values: &mut LastValue) {
    let current_values = proxy_server::get_stats().await;

    let (last_timestamp, last_proxy_stats) = &last_values.proxy;
    let now = Instant::now();

    for (server_id, stats) in &current_values {
        let last_stats = last_proxy_stats.get(server_id);

        let received_bytes = stats.received_bytes;
        let sent_bytes = stats.sent_bytes;

        if let Some(last_stats) = last_stats {
            let delta_received = received_bytes.saturating_sub(last_stats.received_bytes);
            let delta_sent = sent_bytes.saturating_sub(last_stats.sent_bytes);

            let received_rate =
                delta_received as f64 / now.duration_since(*last_timestamp).as_secs_f64();
            let sent_rate = delta_sent as f64 / now.duration_since(*last_timestamp).as_secs_f64();

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let labels = vec![Label::new("server_id", server_id.to_string())];

            metrics.push(Row::with_labels(
                "proxy_received_bytes_per_second",
                labels.clone(),
                DataPoint::new(timestamp, received_rate),
            ));
            metrics.push(Row::with_labels(
                "proxy_sent_bytes_per_second",
                labels.clone(),
                DataPoint::new(timestamp, sent_rate),
            ));
        }
    }

    last_values.proxy = (now, current_values);
}

fn collect_system_metrics(
    metrics: &mut Vec<Row>,
    runner: &MutexGuard<'_, Runner>,
    last_values: &mut LastValue,
) {
    let pids = runner
        .running_servers
        .iter()
        .map(|server| sysinfo::Pid::from_u32(server.pid))
        .collect::<Vec<_>>();

    let (last_timestamp, last_system) = &mut last_values.system;

    last_system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&pids),
        true,
        ProcessRefreshKind::nothing().with_cpu().with_disk_usage(),
    );

    let now = Instant::now();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    for (pid, process) in last_system.processes() {
        let cpu_usage_percent = process.cpu_usage() as f64;

        let server_id = runner
            .running_servers
            .iter()
            .find(|server| sysinfo::Pid::from_u32(server.pid) == *pid)
            .map(|server| server.manifest.id.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let labels = vec![Label::new("server_id", server_id)];

        metrics.push(Row::with_labels(
            "cpu_usage_percent",
            labels.clone(),
            DataPoint::new(timestamp, cpu_usage_percent),
        ));
    }

    *last_timestamp = now;
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
