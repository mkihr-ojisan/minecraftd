use std::time::Duration;

use anyhow::{Context, bail};
use mcctl_protocol::client::Client;
use minecraftd_manifest::ServerManifest;

use crate::cli::{self, UpdateArgs};

pub async fn update(args: UpdateArgs) -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Updating server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let update_type = match args.update_type {
        cli::UpdateType::Stable => mcctl_protocol::UpdateType::Stable,
        cli::UpdateType::Latest => mcctl_protocol::UpdateType::Latest,
    };

    let result = client.update_server(&server_dir, update_type).await?;

    if result.updated {
        pb.finish_with_message(format!(
            "Server updated successfully from version {} build {} to version {} build {}.",
            result.old_version.unwrap_or_default(),
            result.old_build.unwrap_or_default(),
            result.new_version.unwrap_or_default(),
            result.new_build.unwrap_or_default()
        ));
    } else {
        pb.finish_with_message("Server is already up to date.");
    }

    Ok(())
}
