use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const TERMINAL_BUFFER_SIZE: usize = 1024;

#[allow(clippy::large_enum_variant)]
pub enum TerminalInput {
    Input { content: Buffer },
    Resize { cols: u16, rows: u16 },
}

#[derive(Debug, Clone)]
pub enum TerminalOutput {
    Output { content: Buffer },
}

#[derive(Debug, Clone)]
pub struct Buffer {
    len: usize,
    data: [u8; TERMINAL_BUFFER_SIZE],
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            len: 0,
            data: [0; TERMINAL_BUFFER_SIZE],
        }
    }
}

pub struct TerminalReader {
    terminal_out: tokio::sync::broadcast::Receiver<TerminalOutput>,
}

pub struct TerminalWriter {
    terminal_in: tokio::sync::mpsc::Sender<TerminalInput>,
}

impl TerminalReader {
    pub(super) fn new(terminal_out: tokio::sync::broadcast::Receiver<TerminalOutput>) -> Self {
        Self { terminal_out }
    }
}

impl TerminalWriter {
    pub(super) fn new(terminal_in: tokio::sync::mpsc::Sender<TerminalInput>) -> Self {
        Self { terminal_in }
    }
}

impl mcctl_protocol::server::TerminalWriter<anyhow::Error> for TerminalWriter {
    async fn write(&mut self, content: &[u8]) -> anyhow::Result<()> {
        let mut offset = 0;
        while offset < content.len() {
            let chunk_size = std::cmp::min(TERMINAL_BUFFER_SIZE, content.len() - offset);
            let mut buffer = Buffer {
                len: chunk_size,
                ..Default::default()
            };
            buffer.data[..chunk_size].copy_from_slice(&content[offset..offset + chunk_size]);

            self.terminal_in
                .send(TerminalInput::Input { content: buffer })
                .await
                .context("Failed to send terminal input")?;

            offset += chunk_size;
        }
        Ok(())
    }

    async fn resize(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.terminal_in
            .send(TerminalInput::Resize { cols, rows })
            .await
            .context("Failed to send terminal resize")?;
        Ok(())
    }
}

impl mcctl_protocol::server::TerminalReader<anyhow::Error> for TerminalReader {
    async fn read(&mut self) -> anyhow::Result<Option<mcctl_protocol::TerminalOutput>> {
        match self.terminal_out.recv().await {
            Ok(TerminalOutput::Output { content }) => Ok(Some(mcctl_protocol::TerminalOutput {
                content: content.data[..content.len].to_vec(),
            })),
            Err(_) => Ok(None),
        }
    }
}

pub fn spawn_terminal_writer(
    mut pty_writer: pty_process::OwnedWritePty,
    mut term_in_rx: tokio::sync::mpsc::Receiver<TerminalInput>,
) {
    tokio::spawn(async move {
        while let Some(input) = term_in_rx.recv().await {
            match input {
                TerminalInput::Input { content } => {
                    trace!(
                        "Writing to PTY: {}",
                        String::from_utf8_lossy(&content.data[..content.len]).trim()
                    );

                    if let Err(e) = pty_writer.write_all(&content.data[..content.len]).await {
                        eprintln!("Failed to write to PTY: {e}");
                        break;
                    }
                }
                TerminalInput::Resize { cols, rows } => {
                    if let Err(e) = pty_writer.resize(pty_process::Size::new(rows, cols)) {
                        eprintln!("Failed to resize PTY: {e}");
                        break;
                    }
                }
            }
        }
    });
}

pub fn spawn_terminal_reader(
    mut pty_reader: pty_process::OwnedReadPty,
    term_out_tx: tokio::sync::broadcast::Sender<TerminalOutput>,
) {
    tokio::spawn(async move {
        loop {
            let mut buffer = Buffer::default();

            buffer.len = match pty_reader.read(&mut buffer.data).await {
                Ok(0) | Err(_) => break, // I/O error on process exit
                Ok(n) => n,
            };

            trace!(
                "Read from PTY: {}",
                String::from_utf8_lossy(&buffer.data[..buffer.len]).trim()
            );

            let _ = term_out_tx.send(TerminalOutput::Output { content: buffer });
        }
    });
}
