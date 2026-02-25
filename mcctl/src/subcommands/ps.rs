use std::time::Duration;

use anyhow::Context;
use mcctl_protocol::{ServerStatus, client::Client};

pub async fn ps() -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let servers = client.get_running_servers().await?;

    let mut table = Vec::<[String; 6]>::new();
    table.push([
        "NAME".to_string(),
        "STATUS".to_string(),
        "UPTIME".to_string(),
        "PORT".to_string(),
        "PLAYERS".to_string(),
        "DIRECTORY".to_string(),
    ]);

    for server in servers {
        table.push([
            server.name,
            match ServerStatus::try_from(server.status) {
                Ok(ServerStatus::Starting) => "Starting",
                Ok(ServerStatus::Ready) => "Ready",
                Ok(ServerStatus::Stopping) => "Stopping",
                Ok(ServerStatus::Restarting) => "Restarting",
                Err(_) => "Unknown",
            }
            .to_string(),
            match Duration::from_secs(server.uptime_seconds) {
                d if d.as_secs() < 60 => format!("{}s", d.as_secs()),
                d if d.as_secs() < 3600 => format!("{}m{}s", d.as_secs() / 60, d.as_secs() % 60),
                d => format!(
                    "{}h{}m{}s",
                    d.as_secs() / 3600,
                    (d.as_secs() % 3600) / 60,
                    d.as_secs() % 60
                ),
            },
            server.port.to_string(),
            if let Some(player_count) = server.player_count
                && let Some(max_players) = server.max_players
            {
                format!("{}/{}", player_count, max_players)
            } else {
                "-".to_string()
            },
            server.server_dir.to_string(),
        ]);
    }

    let column_widths = (0..6)
        .map(|i| table.iter().map(|row| row[i].len()).max().unwrap_or(0))
        .collect::<Vec<_>>();

    for row in table {
        for (i, cell) in row.iter().enumerate() {
            print!("{:width$}  ", cell, width = column_widths[i]);
        }
        println!();
    }

    Ok(())
}
