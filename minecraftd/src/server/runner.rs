use std::{
    collections::HashMap,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::{Duration, Instant},
};

use anyhow::{Context, bail};
use pty_process::Pty;
use rand::distr::{Alphanumeric, SampleString};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Child,
    sync::Mutex,
    task::JoinSet,
    time::timeout,
};
use uuid::Uuid;

use crate::{
    port::Port,
    server::{
        config::{Connection, ServerConfig},
        server_list_ping::server_list_ping,
        server_properties::ServerProperties,
    },
    util::observable_value::ObservableValue,
};

const STOP_TIMEOUT_SECS: u64 = 180;
const MINECRAFT_DEFAULT_PORT: u16 = 25565;
const PTY_DEFAULT_ROWS: u16 = 24;
const PTY_DEFAULT_COLS: u16 = 80;
const TERMINAL_BUFFER_SIZE: usize = 1024;
const REQUEST_STOP_RETRY_LIMIT: usize = 5;
const REQUEST_STOP_RETRY_INTERVAL_SECS: u64 = 10;

static RUNNER: LazyLock<Mutex<Runner>> = LazyLock::new(|| Mutex::new(Runner::new()));

struct Runner {
    running_servers: RunningServers,
}

struct RunningServers {
    servers: HashMap<Uuid, RunningServer>,
    hostname_to_id: HashMap<String, Uuid>,
    server_dir_to_id: HashMap<PathBuf, Uuid>,
}
impl RunningServers {
    fn new() -> Self {
        Self {
            servers: HashMap::new(),
            hostname_to_id: HashMap::new(),
            server_dir_to_id: HashMap::new(),
        }
    }
    fn insert(&mut self, server: RunningServer) {
        if let Connection::Proxy { hostname } = &server.config.connection {
            self.hostname_to_id.insert(hostname.to_string(), server.id);
        }
        self.server_dir_to_id
            .insert(server.server_dir.clone(), server.id);
        self.servers.insert(server.id, server);
    }
    fn remove(&mut self, id: &Uuid) -> Option<RunningServer> {
        let server = self.servers.remove(id)?;
        if let Connection::Proxy { hostname } = &server.config.connection {
            self.hostname_to_id.remove(hostname);
        }
        self.server_dir_to_id.remove(&server.server_dir);
        Some(server)
    }
    fn contains(&self, id: &Uuid) -> bool {
        self.servers.contains_key(id)
    }
    fn get(&self, id: &Uuid) -> Option<&RunningServer> {
        self.servers.get(id)
    }
    fn get_mut(&mut self, id: &Uuid) -> Option<&mut RunningServer> {
        self.servers.get_mut(id)
    }
    fn get_id_by_hostname(&self, hostname: &str) -> Option<Uuid> {
        self.hostname_to_id.get(hostname).copied()
    }
    fn get_id_by_server_dir(&self, server_dir: &Path) -> anyhow::Result<Option<Uuid>> {
        Ok(self
            .server_dir_to_id
            .get(&server_dir.canonicalize()?)
            .copied())
    }
    fn get_by_server_dir(&self, server_dir: &Path) -> anyhow::Result<Option<&RunningServer>> {
        let id = match self.get_id_by_server_dir(server_dir)? {
            Some(id) => id,
            None => return Ok(None),
        };
        Ok(self.servers.get(&id))
    }
    fn get_mut_by_server_dir(&mut self, server_dir: &Path) -> Option<&mut RunningServer> {
        let id = self.get_id_by_server_dir(server_dir).ok().flatten()?;
        self.servers.get_mut(&id)
    }
}

struct RunningServer {
    id: Uuid,
    server_dir: PathBuf,
    status: ObservableValue<ServerStatus>,
    config: ServerConfig,
    terminal_in: tokio::sync::mpsc::Sender<TerminalInput>,
    terminal_out: tokio::sync::broadcast::Sender<TerminalOutput>,
    server_port: ServerPort,
    rcon_port: Port,
    rcon_password: String,
    pid: u32,
    running_since: Instant,
}

pub struct RunningServerInfo {
    pub server_dir: PathBuf,
    pub name: String,
    pub status: ServerStatus,
    pub server_port: u16,
    pub players: Option<PlayersInfo>,
    pub uptime: Duration,
}

pub struct PlayersInfo {
    pub online: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    Starting {
        restarting: bool,
    },
    Ready,
    Stopping {
        restarting: bool,
    },
    /// Used only for waiting for server to stop using wait_for_server_status.
    /// Stopped servers are immediately removed from the list.
    Stopped,
}

#[allow(clippy::large_enum_variant)]
enum TerminalInput {
    Input { content: Buffer },
    Resize { cols: u16, rows: u16 },
}

#[derive(Debug, Clone)]
enum TerminalOutput {
    Output { content: Buffer },
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub len: usize,
    pub data: [u8; TERMINAL_BUFFER_SIZE],
}
impl Default for Buffer {
    fn default() -> Self {
        Self {
            len: 0,
            data: [0; TERMINAL_BUFFER_SIZE],
        }
    }
}

enum ServerPort {
    Proxy(Port),
    Direct(u16),
}
impl ServerPort {
    fn port(&self) -> u16 {
        match self {
            ServerPort::Proxy(p) => p.port(),
            ServerPort::Direct(p) => *p,
        }
    }
}

pub struct TerminalWriter {
    terminal_in: tokio::sync::mpsc::Sender<TerminalInput>,
}

pub struct TerminalReader {
    terminal_out: tokio::sync::broadcast::Receiver<TerminalOutput>,
}

impl Runner {
    fn new() -> Self {
        Self {
            running_servers: RunningServers::new(),
        }
    }
}

pub async fn get_server_id_by_hostname(hostname: &str) -> Option<Uuid> {
    let runner = RUNNER.lock().await;
    runner.running_servers.get_id_by_hostname(hostname)
}

/// Never returns ServerStatus::Stopped
pub async fn get_server_status(id: Uuid) -> Option<ServerStatus> {
    let runner = RUNNER.lock().await;
    runner.running_servers.get(&id).map(|s| s.status.get())
}

pub async fn get_server_port(id: Uuid) -> Option<u16> {
    let runner = RUNNER.lock().await;
    runner
        .running_servers
        .get(&id)
        .map(|s| s.server_port.port())
}

pub async fn get_running_servers() -> Vec<RunningServerInfo> {
    let runner = RUNNER.lock().await;
    let mut servers = Vec::new();

    for server in runner.running_servers.servers.values() {
        let players = match server.status.get() {
            ServerStatus::Ready => {
                server_list_ping((Ipv4Addr::LOCALHOST, server.server_port.port()))
                    .await
                    .ok()
                    .and_then(|ping| {
                        if let Some(players) = ping.players {
                            Some(PlayersInfo {
                                online: players.online as u32,
                                max: players.max as u32,
                            })
                        } else {
                            None
                        }
                    })
            }
            _ => None,
        };

        servers.push(RunningServerInfo {
            server_dir: server.server_dir.clone(),
            name: server.config.name.clone(),
            status: server.status.get(),
            server_port: server.server_port.port(),
            players,
            uptime: server.running_since.elapsed(),
        });
    }

    servers
}

pub async fn start_server(server_dir: &Path) -> anyhow::Result<()> {
    do_start_server(server_dir, false).await
}

pub async fn stop_server(server_dir: &Path) -> anyhow::Result<()> {
    let id;
    {
        let runner = RUNNER.lock().await;
        let Some(server) = runner.running_servers.get_by_server_dir(server_dir)? else {
            bail!("Server at '{}' is not running", server_dir.display());
        };
        id = server.id;
    }

    do_stop_server(id, false).await
}

pub async fn kill_server(server_dir: &Path) -> anyhow::Result<()> {
    let id;
    {
        let runner = RUNNER.lock().await;
        let Some(server) = runner.running_servers.get_by_server_dir(server_dir)? else {
            bail!("Server at '{}' is not running", server_dir.display());
        };
        id = server.id;
    }

    do_kill_server(id).await
}

pub async fn attach_terminal(
    server_dir: &Path,
) -> anyhow::Result<(TerminalReader, TerminalWriter)> {
    let mut runner = RUNNER.lock().await;
    let Some(server) = runner.running_servers.get_mut_by_server_dir(server_dir) else {
        bail!("Server at '{}' is not running", server_dir.display());
    };

    let terminal_in = server.terminal_in.clone();
    let terminal_out = server.terminal_out.subscribe();
    Ok((
        TerminalReader { terminal_out },
        TerminalWriter { terminal_in },
    ))
}

pub async fn wait_ready(server_dir: &Path) -> anyhow::Result<()> {
    let id;
    {
        let runner = RUNNER.lock().await;
        let Some(server) = runner.running_servers.get_by_server_dir(server_dir)? else {
            bail!("Server at '{}' is not running", server_dir.display());
        };
        id = server.id;
    }

    wait_for_server_status(id, ServerStatus::Ready).await
}

pub async fn restart_server(server_dir: &Path) -> anyhow::Result<()> {
    let id = {
        let runner = RUNNER.lock().await;
        let Some(server) = runner.running_servers.get_by_server_dir(server_dir)? else {
            bail!("Server at '{}' is not running", server_dir.display());
        };
        server.id
    };

    do_stop_server(id, true).await?;
    do_start_server(server_dir, true).await
}

pub async fn shutdown() {
    let mut join_set = JoinSet::new();

    {
        let runner = RUNNER.lock().await;
        for server in runner.running_servers.servers.values() {
            let server_id = server.id;
            join_set.spawn(async move {
                if let Err(e) = do_stop_server(server_id, false).await {
                    error!("Failed to stop server '{server_id}': {e:?}",);
                }
            });
        }
    }

    join_set.join_all().await;
}

async fn do_start_server(server_dir: &Path, restarting: bool) -> anyhow::Result<()> {
    let mut runner = RUNNER.lock().await;

    let server_dir = server_dir.canonicalize()?;
    if runner
        .running_servers
        .get_id_by_server_dir(&server_dir)?
        .is_some()
    {
        bail!("Server at '{}' is already running", server_dir.display());
    }

    info!("Starting server at '{}'", server_dir.display());

    let id = loop {
        let id = Uuid::new_v4();
        if !runner.running_servers.contains(&id) {
            break id;
        }
    };
    debug!("Assigned server ID: {}", id);

    let config = ServerConfig::load(&server_dir).await?;
    debug!("Loaded server config: {:?}", config);

    if let Connection::Proxy { hostname } = &config.connection
        && runner
            .running_servers
            .get_id_by_hostname(hostname)
            .is_some()
    {
        bail!("A server with hostname '{}' is already running", hostname);
    }

    let (server_port, rcon_port, rcon_password) = prepare_server(&server_dir, &config).await?;

    config.java_runtime.prepare().await?;

    let java_path = config.java_runtime.java_path();
    let command_args_str = command_substitute_placeholders(
        &config.command,
        java_path.to_str().context("java_path is not valid UTF-8")?,
    );
    let (pty, child) = start_command_with_pty(&command_args_str, &server_dir)?;
    let pid = child.id().context("Failed to get child process ID")?;

    let (pty_reader, pty_writer) = pty.into_split();
    let (term_in_tx, term_in_rx) = tokio::sync::mpsc::channel::<TerminalInput>(1);
    let (term_out_tx, _) = tokio::sync::broadcast::channel::<TerminalOutput>(16);
    spawn_terminal_writer(pty_writer, term_in_rx);
    spawn_terminal_reader(pty_reader, term_out_tx.clone());

    spawn_process_watcher(id, child);

    spawn_readiness_checker(id, server_port.port());

    runner.running_servers.insert(RunningServer {
        id,
        server_dir,
        status: ObservableValue::new(ServerStatus::Starting { restarting }),
        config,
        terminal_in: term_in_tx,
        terminal_out: term_out_tx,
        server_port,
        rcon_port,
        rcon_password,
        pid,
        running_since: Instant::now(),
    });

    Ok(())
}

async fn prepare_server(
    server_dir: &Path,
    config: &ServerConfig,
) -> anyhow::Result<(ServerPort, Port, String)> {
    let mut server_properties = ServerProperties::load(server_dir).await.unwrap_or_default();

    let server_port = if let Connection::Proxy { .. } = &config.connection {
        let server_port = Port::acquire()?;
        server_properties.set("server-port", server_port.port().to_string());
        ServerPort::Proxy(server_port)
    } else {
        let port = server_properties
            .get("server-port")
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(MINECRAFT_DEFAULT_PORT);
        ServerPort::Direct(port)
    };

    let rcon_port = Port::acquire()?;
    server_properties.set("enable-rcon", "true");
    server_properties.set("rcon.port", rcon_port.port().to_string());
    let rcon_password = match server_properties.get("rcon.password") {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => {
            let rcon_password = Alphanumeric.sample_string(&mut rand::rng(), 16);
            server_properties.set("rcon.password", &rcon_password);
            rcon_password
        }
    };

    server_properties.save(server_dir).await?;

    debug!(
        "Prepared server properties with server_port={}, rcon_port={}",
        server_port.port(),
        rcon_port.port()
    );

    Ok((server_port, rcon_port, rcon_password))
}

fn command_substitute_placeholders(command: &[String], java_path: &str) -> Vec<String> {
    command
        .iter()
        .map(|part| part.replace("${java}", java_path))
        .collect()
}

fn start_command_with_pty(
    command_args_str: &[String],
    server_dir: &Path,
) -> anyhow::Result<(Pty, Child)> {
    debug!("Starting command: {:?}", command_args_str);

    let command_str = command_args_str.first().context("command is empty")?;
    let args_str = &command_args_str[1..];

    let command = pty_process::Command::new(command_str)
        .args(args_str)
        .current_dir(server_dir)
        .kill_on_drop(true);

    let (pty, pts) = pty_process::open().context("Failed to open PTY")?;
    pty.resize(pty_process::Size::new(PTY_DEFAULT_ROWS, PTY_DEFAULT_COLS))
        .context("Failed to set PTY size")?;

    let child = command
        .spawn(pts)
        .context("Failed to spawn server process")?;

    debug!(
        "Spawned server process with PID {}",
        child.id().unwrap_or(0)
    );

    Ok((pty, child))
}

fn spawn_terminal_writer(
    mut pty_writer: pty_process::OwnedWritePty,
    mut term_in_rx: tokio::sync::mpsc::Receiver<TerminalInput>,
) {
    tokio::spawn(async move {
        while let Some(input) = term_in_rx.recv().await {
            match input {
                TerminalInput::Input { content } => {
                    trace!(
                        "Writing to PTY: {}",
                        String::from_utf8_lossy(&content.data[..content.len])
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

fn spawn_terminal_reader(
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
                String::from_utf8_lossy(&buffer.data[..buffer.len])
            );

            let _ = term_out_tx.send(TerminalOutput::Output { content: buffer });
        }
    });
}

fn spawn_process_watcher(id: Uuid, mut process: Child) {
    tokio::spawn(async move {
        match process.wait().await {
            Ok(status) => {
                info!("Server process exited with status: {}", status);

                let mut runner = RUNNER.lock().await;
                runner
                    .running_servers
                    .get(&id)
                    .expect("Server should exist")
                    .status
                    .set(ServerStatus::Stopped);
                runner.running_servers.remove(&id);
            }
            Err(e) => {
                error!("Failed to wait for server process: {:?}", e);
            }
        }
    });
}

fn spawn_readiness_checker(id: Uuid, server_port: u16) {
    tokio::spawn(async move {
        let server_addr = (Ipv4Addr::LOCALHOST, server_port);

        macro_rules! is_ready {
            () => {
                match timeout(Duration::from_secs(10), server_list_ping(server_addr)).await {
                    Ok(Ok(_)) => true,
                    _ => false,
                }
            };
        }

        while !is_ready!() {
            tokio::time::sleep(Duration::from_secs(1)).await;

            if let Some(ServerStatus::Starting { .. }) = get_server_status(id).await {
            } else {
                debug!("Server {id} is no longer starting, aborting readiness check",);
                return;
            }
        }

        {
            let mut runner = RUNNER.lock().await;
            let Some(server) = runner.running_servers.get_mut(&id) else {
                return;
            };
            server.status.set(ServerStatus::Ready);
        }

        info!("Server {id} is now ready");
    });
}

async fn do_stop_server(id: Uuid, restarting: bool) -> anyhow::Result<()> {
    {
        let mut runner = RUNNER.lock().await;

        let Some(server) = runner.running_servers.get_mut(&id) else {
            bail!("Server is not running");
        };

        server.status.set(ServerStatus::Stopping { restarting });

        if let Err(err) = request_server_stop(server.rcon_port.port(), &server.rcon_password).await
        {
            debug!(
                "Failed to request server stop. Killing it instead: {:?}",
                err
            );
            do_kill_server(id).await?;
        }
    }

    let wait = wait_for_server_status(id, ServerStatus::Stopped);

    match timeout(Duration::from_secs(STOP_TIMEOUT_SECS), wait).await {
        Ok(_) => {
            debug!("Server stopped successfully");
            Ok(())
        }
        Err(_) => {
            warn!("Server did not stop within {STOP_TIMEOUT_SECS} seconds, killing it");
            do_kill_server(id).await
        }
    }
}

async fn request_server_stop(rcon_port: u16, rcon_password: &str) -> anyhow::Result<()> {
    let mut count = 0;
    loop {
        let result: anyhow::Result<()> = async {
            let mut client =
                minecraft_rcon::Client::connect((Ipv4Addr::LOCALHOST, rcon_port), rcon_password)
                    .await
                    .context("Failed to connect to RCON")?;

            client
                .execute_command("stop")
                .await
                .context("Failed to send stop command")?;

            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                debug!("Sent stop command successfully");
                return Ok(());
            }
            Err(e) => {
                if count >= REQUEST_STOP_RETRY_LIMIT {
                    bail!(
                        "Failed to send stop command after {REQUEST_STOP_RETRY_LIMIT} attempts: {e}"
                    );
                }

                warn!(
                    "Failed to send stop command. Retrying... (attempt {}): {:?}",
                    count + 1,
                    e
                );
                tokio::time::sleep(Duration::from_secs(REQUEST_STOP_RETRY_INTERVAL_SECS)).await;
                count += 1;
            }
        }
    }
}

async fn wait_for_server_status(id: Uuid, desired_status: ServerStatus) -> anyhow::Result<()> {
    let waiter = {
        let runner = RUNNER.lock().await;
        let Some(server) = runner.running_servers.get(&id) else {
            bail!("Server is not running");
        };

        server.status.wait_until(move |s| *s == desired_status)
    };

    let result = waiter.await;
    if desired_status == ServerStatus::Stopped {
        Ok(())
    } else {
        result.context("Server stopped")
    }
}

async fn do_kill_server(id: Uuid) -> anyhow::Result<()> {
    let runner = RUNNER.lock().await;
    let Some(server) = runner.running_servers.get(&id) else {
        bail!("Server is not running");
    };
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(server.pid as i32),
        nix::sys::signal::Signal::SIGKILL,
    )?;
    Ok(())
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
