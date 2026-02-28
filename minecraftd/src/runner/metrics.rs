use std::time::SystemTime;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};

use crate::{
    metrics::{self, MetricsCollector, MetricsCollectorContext},
    runner::RUNNER,
    util::BoxedFuture,
};

pub async fn init_metrics() -> anyhow::Result<()> {
    metrics::register_metrics_collector(BridgeMetricsCollector).await;
    metrics::register_metrics_collector(SystemMetricsCollector::default()).await;

    Ok(())
}

struct BridgeMetricsCollector;
impl MetricsCollector for BridgeMetricsCollector {
    fn name(&self) -> &'static str {
        "bridge_metrics_collector"
    }

    fn collect<'a>(
        &'a mut self,
        ctx: &'a mut MetricsCollectorContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let runner = RUNNER.lock().await;

            for server in runner.running_servers.iter() {
                let Some(bridge) = server.bridge.get() else {
                    continue;
                };

                let mut bridge = bridge.lock().await;

                let timestamp = SystemTime::now();

                match bridge.get_server_metrics().await {
                    Ok(server_metrics) => {
                        if let Some(tps) = server_metrics.tps {
                            ctx.push_metric(server.manifest.id, "tps", timestamp, tps);
                        }
                        if let Some(mspt) = server_metrics.mspt {
                            ctx.push_metric(server.manifest.id, "mspt", timestamp, mspt);
                        }
                        if let Some(player_count) = server_metrics.player_count {
                            ctx.push_metric(
                                server.manifest.id,
                                "player_count",
                                timestamp,
                                player_count as f64,
                            );
                        }
                        if let Some(entity_count) = server_metrics.entity_count {
                            ctx.push_metric(
                                server.manifest.id,
                                "entity_count",
                                timestamp,
                                entity_count as f64,
                            );
                        }
                        if let Some(loaded_chunk_count) = server_metrics.loaded_chunk_count {
                            ctx.push_metric(
                                server.manifest.id,
                                "loaded_chunk_count",
                                timestamp,
                                loaded_chunk_count as f64,
                            );
                        }
                        if let Some(allocated_memory) = server_metrics.allocated_memory {
                            ctx.push_metric(
                                server.manifest.id,
                                "allocated_memory_bytes",
                                timestamp,
                                allocated_memory as f64,
                            );
                        }
                        if let Some(used_memory) = server_metrics.used_memory {
                            ctx.push_metric(
                                server.manifest.id,
                                "used_memory_bytes",
                                timestamp,
                                used_memory as f64,
                            );
                        }
                    }
                    Err(e) => error!(
                        "Failed to collect metrics for server {}: {:?}",
                        server.manifest.id, e
                    ),
                }
            }

            Ok(())
        })
    }
}

#[derive(Default)]
struct SystemMetricsCollector {
    system: sysinfo::System,
}

impl MetricsCollector for SystemMetricsCollector {
    fn name(&self) -> &'static str {
        "system_metrics_collector"
    }

    fn collect<'a>(
        &'a mut self,
        ctx: &'a mut MetricsCollectorContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let runner = RUNNER.lock().await;

            let pids = runner
                .running_servers
                .iter()
                .map(|server| sysinfo::Pid::from_u32(server.pid))
                .collect::<Vec<_>>();

            self.system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&pids),
                true,
                ProcessRefreshKind::nothing().with_cpu().with_disk_usage(),
            );

            let timestamp = SystemTime::now();

            for (pid, process) in self.system.processes() {
                let cpu_usage_percent = process.cpu_usage() as f64;

                let Some(server_id) = runner
                    .running_servers
                    .iter()
                    .find(|server| sysinfo::Pid::from_u32(server.pid) == *pid)
                    .map(|server| server.manifest.id)
                else {
                    continue;
                };

                ctx.push_metric(server_id, "cpu_usage_percent", timestamp, cpu_usage_percent);
            }

            Ok(())
        })
    }
}
