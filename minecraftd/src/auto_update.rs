use tokio::task::JoinSet;

use crate::{
    config::get_config, extension::providers::get_extension_provider, runner,
    server_implementations::get_server_implementation,
};

pub fn init() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(get_config().auto_update.update_check_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            if let Err(err) = do_auto_update().await {
                error!("Auto-update error: {err:?}");
            }
        }
    });
}

async fn do_auto_update() -> anyhow::Result<()> {
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

        let mut update_available = false;

        if let Some((version, build)) = server_implementation
            .is_newer_version_available(&manifest.version, &manifest.build, true)
            .await?
        {
            debug!(
                "New version available for server {id} (version: {}, build: {})",
                version.name, build.name
            );
            update_available = true;
        };

        for extension in &manifest.extensions {
            let extension_provider = get_extension_provider(&extension.provider)
                .expect("Extension provider should exist since the server is running");
            if let Some(new_version) = extension_provider
                .is_newer_version_available(
                    extension.type_,
                    &manifest.version,
                    &extension.id,
                    &extension.version_id,
                )
                .await
                .unwrap_or(None)
            {
                debug!(
                    "New version available for extension {} (version: {})",
                    extension.name, new_version.version
                );
                update_available = true;
            }
        }

        if !update_available {
            debug!("No updates available for server {id}");
            continue;
        }

        drop(manifest);

        join_set.spawn(async move {
            let result: anyhow::Result<()> = async move {
                if tokio::time::timeout(
                    get_config()
                        .auto_update
                        .wait_until_all_players_log_out_timeout,
                    runner::wait_until_all_players_log_out(id),
                )
                .await
                .is_err()
                {
                    let message = get_config().messages.server_restarting_for_update.clone();
                    runner::tellraw(id, "@a", message).await?;
                    tokio::time::sleep(
                        get_config()
                            .auto_update
                            .notify_players_before_restart_interval,
                    )
                    .await;
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
