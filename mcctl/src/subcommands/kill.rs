use std::time::Duration;

use anyhow::{Context, bail};
use mcctl_protocol::client::Client;
use minecraftd_manifest::ServerManifest;

use crate::cli::KillArgs;

pub async fn kill(args: KillArgs) -> anyhow::Result<()> {
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
    pb.set_message("Killing server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    client.kill_server(server_dir).await?;

    pb.finish_with_message("Server killed successfully.");

    Ok(())
}
