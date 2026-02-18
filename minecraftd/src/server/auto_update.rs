use std::time::Duration;

use minecraft_protocol::text_component::{Color, Object, TextComponent};
use tokio::task::JoinSet;

use crate::server::{implementations::get_server_implementation, runner};

const UPDATE_CHECK_INTERVAL: Duration = Duration::from_hours(24);
const WAIT_UNTIL_ALL_PLAYERS_LOG_OUT_TIMEOUT: Duration = Duration::from_hours(1);
const NOTIFY_PLAYERS_BEFORE_RESTART_INTERVAL_MINUTES: u64 = 1;

pub fn start() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(UPDATE_CHECK_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            if let Err(err) = do_update().await {
                error!("Auto-update error: {err:?}");
            }
        }
    });
}

async fn do_update() -> anyhow::Result<()> {
    let servers = runner::get_running_server_ids().await;

    let mut join_set = JoinSet::new();

    for id in servers {
        let Some(server_dir) = runner::get_server_dir(id).await else {
            continue;
        };

        let Some(manifest) = runner::get_server_manifest(id).await else {
            continue;
        };

        if !manifest.auto_update {
            continue;
        }

        let server_implementation = get_server_implementation(&manifest.server_implementation)
            .expect("Server implementation should exist since the server is running");

        debug!("Checking for updates for server {id}");

        let Some((version, build)) = server_implementation
            .is_newer_version_available(&manifest.version, &manifest.build, true)
            .await?
        else {
            continue;
        };

        drop(manifest);

        debug!(
            "New version available for server {id} (version: {}, build: {})",
            version.name, build.name
        );

        join_set.spawn(async move {
            let result: anyhow::Result<()> = async move {
                if tokio::time::timeout(
                    WAIT_UNTIL_ALL_PLAYERS_LOG_OUT_TIMEOUT,
                    runner::wait_until_all_players_log_out(id),
                )
                .await.is_err() {
                    let message = TextComponent::Object(Object {
                        text: Some("".to_string()),
                        extra: Some(vec![
                            TextComponent::Object(Object {
                                text: Some("=== SERVER RESTART ===".to_string()),
                                color: Some(Color::Red),
                                ..Default::default()
                            }),
                            TextComponent::String(format!(
                            "\nServer will restart in {} minute(s) to apply updates.\nPlease log out to avoid interruption.",
                            NOTIFY_PLAYERS_BEFORE_RESTART_INTERVAL_MINUTES
                        ))]),
                        ..Default::default()
                    });
                    runner::tellraw(id, "@a", message).await?;
                    tokio::time::sleep(Duration::from_mins(NOTIFY_PLAYERS_BEFORE_RESTART_INTERVAL_MINUTES)).await;
                }
                runner::restart_server(&server_dir).await?;
                Ok(())
            }
            .await;

            if let Err(err) = result {
                error!("{err:?}");
            }
        });
    }

    join_set.join_all().await;

    Ok(())
}
