use anyhow::{Context, bail};
use mcctl_protocol::client::Client;
use minecraftd_manifest::ServerManifest;
use nix::sys::termios::{LocalFlags, SetArg, tcgetattr, tcsetattr};
use terminal_size::{Height, Width, terminal_size};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::cli::AttachArgs;

pub async fn attach(args: AttachArgs) -> anyhow::Result<()> {
    let client = Client::connect()
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

    let (mut terminal_reader, mut terminal_writer) = client.attach_terminal(server_dir).await?;

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    let mut attr = tcgetattr(&stdin).context("Failed to get terminal attributes")?;
    let orig_attr = attr.clone();
    attr.local_flags
        .remove(LocalFlags::ICANON | LocalFlags::ECHO);
    tcsetattr(&stdin, SetArg::TCSANOW, &attr).context("Failed to set terminal attributes")?;

    let (exit_signal_tx, mut exit_signal_rx) = tokio::sync::mpsc::channel::<()>(1);

    let input_task = tokio::spawn({
        async move {
            let result: anyhow::Result<()> = async {
                let mut buffer = [0u8; 1024];
                let mut sigwinch =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::window_change())
                        .context("Failed to create SIGWINCH signal stream")?;

                loop {
                    tokio::select! {
                        result = stdin.read(&mut buffer) => {
                            let n = result.context("Failed to read from stdin")?;
                            if n == 0 {
                                break;
                            }
                            terminal_writer
                                .write(buffer[..n].to_vec())
                                .await
                                .context("Failed to send terminal input")?;
                        }
                        _ = sigwinch.recv() => {
                            let (Width(cols), Height(rows)) =
                                terminal_size().context("Failed to get terminal size")?;
                            terminal_writer
                                .resize(cols as u32, rows as u32)
                                .await
                                .context("Failed to send resize command")?;
                        }
                        _ = exit_signal_rx.recv() => {
                            break;
                        }
                    };
                }
                Ok(())
            }
            .await;

            if let Err(e) = result {
                eprintln!("Error in input task: {:?}", e);
            }
        }
    });

    let output_task = tokio::spawn({
        async move {
            let result: anyhow::Result<()> = async {
                loop {
                    let Some(output) = terminal_reader
                        .read()
                        .await
                        .context("Failed to read from terminal")?
                    else {
                        exit_signal_tx.send(()).await.ok();
                        break Ok(());
                    };

                    stdout
                        .write_all(&output.content)
                        .await
                        .context("Failed to write to stdout")?;
                    stdout.flush().await.context("Failed to flush stdout")?;
                }
            }
            .await;

            if let Err(e) = result {
                eprintln!("Error in output task: {:?}", e);
            }
        }
    });

    tokio::try_join!(input_task, output_task).unwrap();

    tcsetattr(tokio::io::stdin(), SetArg::TCSANOW, &orig_attr)
        .context("Failed to restore terminal attributes")?;

    Ok(())
}
