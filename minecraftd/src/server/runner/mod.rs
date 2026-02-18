use std::{
    ffi::{OsStr, OsString},
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::{Duration, Instant},
};

use anyhow::{Context, bail};
use minecraft_protocol::text_component::TextComponent;
use minecraftd_manifest::{Connection, ExtensionType, ServerManifest};
use pty_process::Pty;
use rand::distr::{Alphanumeric, SampleString};
use tokio::{
    process::Child,
    sync::{MappedMutexGuard, Mutex, MutexGuard},
    task::JoinSet,
    time::timeout,
};
use uuid::Uuid;

use crate::{
    port::Port,
    server::{
        extension_providers::{extension_cache_root_dir, get_extension_provider},
        implementations::{ServerImplementation, get_server_implementation},
        java_runtime::JavaRuntimeExt,
        runner::{
            auto_start::{
                add_auto_start_directory, get_auto_start_directories, remove_auto_start_directory,
            },
            running_servers::RunningServers,
            terminal::{
                TerminalInput, TerminalOutput, spawn_terminal_reader, spawn_terminal_writer,
            },
        },
        server_list_ping::server_list_ping,
        server_properties::ServerProperties,
    },
    util::{observable_value::ObservableValue, os_str_ext::OsStrExt},
};

pub use terminal::{TerminalReader, TerminalWriter};

mod auto_start;
mod running_servers;
mod terminal;

const STOP_TIMEOUT_SECS: u64 = 180;
const MINECRAFT_DEFAULT_PORT: u16 = 25565;
const PTY_DEFAULT_ROWS: u16 = 24;
const PTY_DEFAULT_COLS: u16 = 80;
const REQUEST_STOP_RETRY_LIMIT: usize = 5;
const REQUEST_STOP_RETRY_INTERVAL_SECS: u64 = 10;
const WAIT_FOR_PLAYER_LOGOUT_INTERVAL_SECS: u64 = 60;

static RUNNER: LazyLock<Mutex<Runner>> = LazyLock::new(|| Mutex::new(Runner::new()));

struct Runner {
    running_servers: RunningServers,
}

struct RunningServer {
    id: Uuid,
    server_dir: PathBuf,
    status: ObservableValue<ServerStatus>,
    manifest: ServerManifest,
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

pub async fn get_server_dir(id: Uuid) -> Option<PathBuf> {
    let runner = RUNNER.lock().await;
    runner
        .running_servers
        .get(&id)
        .map(|s| s.server_dir.clone())
}

pub async fn get_server_port(id: Uuid) -> Option<u16> {
    let runner = RUNNER.lock().await;
    runner
        .running_servers
        .get(&id)
        .map(|s| s.server_port.port())
}

pub async fn is_server_running(server_dir: &Path) -> anyhow::Result<bool> {
    let runner = RUNNER.lock().await;
    Ok(runner
        .running_servers
        .get_id_by_server_dir(server_dir)?
        .is_some())
}

pub async fn get_running_servers() -> Vec<RunningServerInfo> {
    let runner = RUNNER.lock().await;
    let mut servers = Vec::new();

    for server in runner.running_servers.iter() {
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
            name: server.manifest.name.clone(),
            status: server.status.get(),
            server_port: server.server_port.port(),
            players,
            uptime: server.running_since.elapsed(),
        });
    }

    servers
}

pub async fn get_running_server_ids() -> Vec<Uuid> {
    let runner = RUNNER.lock().await;
    runner.running_servers.iter().map(|s| s.id).collect()
}

pub async fn start_server(server_dir: &Path) -> anyhow::Result<()> {
    do_start_server(server_dir, false, false).await
}

pub async fn stop_server(server_dir: &Path) -> anyhow::Result<()> {
    let id;
    {
        let runner = RUNNER.lock().await;
        let Some(server) = runner.running_servers.get_by_server_dir(server_dir)? else {
            bail!("Server at '{}' is not running", server_dir.display());
        };
        id = server.id;

        if server.manifest.auto_start {
            info!(
                "Removing server at '{}' from auto-start list",
                server_dir.display()
            );
            remove_auto_start_directory(&server.server_dir).await?;
        }
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
        TerminalReader::new(terminal_out),
        TerminalWriter::new(terminal_in),
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
    do_start_server(server_dir, true, false).await
}

pub async fn start_auto_start_servers() {
    let auto_start_servers = get_auto_start_directories().await;

    info!("Auto-starting servers: {:?}", auto_start_servers);

    for server_dir in auto_start_servers {
        tokio::spawn(async move {
            if let Err(e) = do_start_server(&server_dir, false, true).await {
                error!(
                    "Failed to auto-start server at '{}': {:?}",
                    server_dir.display(),
                    e
                );
                if let Err(e) = remove_auto_start_directory(&server_dir).await {
                    error!("{e:?}",);
                }
            }
        });
    }
}

pub async fn shutdown() {
    let mut join_set = JoinSet::new();

    {
        let runner = RUNNER.lock().await;
        for server in runner.running_servers.iter() {
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

pub async fn wait_until_all_players_log_out(id: Uuid) -> anyhow::Result<()> {
    loop {
        {
            let runner = RUNNER.lock().await;

            let server = runner
                .running_servers
                .get(&id)
                .context("Server is not running")?;

            if server.status.get() == ServerStatus::Ready {
                if let Some(players) =
                    server_list_ping((Ipv4Addr::LOCALHOST, server.server_port.port()))
                        .await
                        .ok()
                        .and_then(|ping| ping.players)
                    && players.online == 0
                {
                    return Ok(());
                }
            } else {
                bail!("Server is no longer in ready state");
            }
        }

        tokio::time::sleep(Duration::from_secs(WAIT_FOR_PLAYER_LOGOUT_INTERVAL_SECS)).await;
    }
}

pub async fn get_server_manifest(id: Uuid) -> Option<MappedMutexGuard<'static, ServerManifest>> {
    let runner = RUNNER.lock().await;

    runner.running_servers.get(&id)?;

    Some(MutexGuard::map(runner, move |r| {
        let server = r
            .running_servers
            .get_mut(&id)
            .expect("Server should exist since we have the lock");
        &mut server.manifest
    }))
}

pub async fn tellraw(id: Uuid, target: &str, message: TextComponent) -> anyhow::Result<()> {
    let runner = RUNNER.lock().await;

    let server = runner
        .running_servers
        .get(&id)
        .context("Server is not running")?;

    if server.status.get() != ServerStatus::Ready {
        bail!("Server is not in ready state");
    }

    let mut rcon_client = minecraft_rcon::Client::connect(
        (Ipv4Addr::LOCALHOST, server.rcon_port.port()),
        &server.rcon_password,
    )
    .await?;

    rcon_client
        .execute_command(&format!(
            "tellraw {} {}",
            target,
            serde_json::to_string(&message)?
        ))
        .await?;

    Ok(())
}

async fn do_start_server(
    server_dir: &Path,
    restarting: bool,
    auto_starting: bool,
) -> anyhow::Result<()> {
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

    let mut manifest = ServerManifest::load(&server_dir).await?;
    debug!("Loaded server manifest: {:?}", manifest);

    let server_implementation = get_server_implementation(&manifest.server_implementation)
        .with_context(|| {
            format!(
                "Unknown server implementation '{}'",
                manifest.server_implementation
            )
        })?;

    if manifest.auto_update {
        update_server_if_newer_version_is_available(
            &server_dir,
            server_implementation,
            &mut manifest,
        )
        .await?;
    }

    if let Connection::Proxy { hostname } = &manifest.connection
        && runner
            .running_servers
            .get_id_by_hostname(hostname)
            .is_some()
    {
        bail!("A server with hostname '{}' is already running", hostname);
    }

    if manifest.auto_start {
        add_auto_start_directory(&server_dir).await?;
        info!(
            "Auto-start is enabled for server at '{}'",
            server_dir.display()
        );
    } else {
        remove_auto_start_directory(&server_dir).await?;
        info!(
            "Auto-start is disabled for server at '{}'",
            server_dir.display()
        );
        if auto_starting {
            info!(
                "Server at '{}' is not set to auto-start. Skipping.",
                server_dir.display()
            );
            return Ok(());
        }
    }

    let (server_port, rcon_port, rcon_password) =
        prepare_server_properties(&server_dir, &manifest).await?;

    prepare_extensions(&server_dir, &manifest).await?;

    manifest.java_runtime.prepare().await?;

    let java_path = manifest.java_runtime.java_path();

    let server_jar_path = server_implementation
        .get_server_jar_path(&server_dir, &manifest.version, &manifest.build)
        .await
        .context("Failed to prepare server jar")?;

    let command_args_str =
        command_substitute_placeholders(&manifest.command, &java_path, &server_jar_path);
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
        manifest,
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

async fn update_server_if_newer_version_is_available(
    server_dir: &Path,
    server_implementation: &dyn ServerImplementation,
    manifest: &mut ServerManifest,
) -> anyhow::Result<()> {
    if let Some((version, build)) = server_implementation
        .is_newer_version_available(&manifest.version, &manifest.build, false)
        .await?
    {
        info!(
            "New version '{}' build '{}' is available for server implementation '{}'. Updating manifest.",
            version.name, build.name, manifest.server_implementation
        );
        manifest.version = version.name;
        manifest.build = build.name;

        manifest.save(server_dir).await.with_context(|| {
            format!(
                "Failed to save updated manifest for server at '{}'",
                server_dir.display()
            )
        })?;
    }

    Ok(())
}

async fn prepare_server_properties(
    server_dir: &Path,
    manifest: &ServerManifest,
) -> anyhow::Result<(ServerPort, Port, String)> {
    let mut server_properties = ServerProperties::load(server_dir).await.unwrap_or_default();

    let server_port = if let Connection::Proxy { .. } = &manifest.connection {
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

async fn prepare_extensions(server_dir: &Path, manifest: &ServerManifest) -> anyhow::Result<()> {
    let extension_cache_dir = extension_cache_root_dir()?;

    let mods_dir = server_dir.join("mods");
    let plugins_dir = server_dir.join("plugins");

    let types = [
        (ExtensionType::Mod, &mods_dir),
        (ExtensionType::Plugin, &plugins_dir),
    ];

    // symlink path, type, provider name, id, version id
    let mut managed_mods_in_mods_dir =
        Vec::<(PathBuf, ExtensionType, OsString, OsString, OsString)>::new();

    // find all symlinks in mods and plugins directories and check if they are managed by us
    for (type_, dir) in types {
        if !dir.exists() {
            continue;
        }

        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_symlink() {
                let target = tokio::fs::read_link(&path).await?;
                if let Ok(relative_target) = target.strip_prefix(&extension_cache_dir) {
                    let components = relative_target.components().collect::<Vec<_>>();
                    if components.len() == 5 {
                        let provider = components[0].as_os_str().to_os_string();

                        let ty = match components[1].as_os_str().to_str() {
                            Some("mods") => ExtensionType::Mod,
                            Some("plugins") => ExtensionType::Plugin,
                            _ => continue,
                        };

                        if ty != type_ {
                            warn!(
                                "Symlink at '{}' points to a {:?} but is in the {:?} directory. Removing it.",
                                path.display(),
                                ty,
                                type_
                            );
                            tokio::fs::remove_file(&path).await?;
                            continue;
                        }

                        let id = components[2].as_os_str().to_os_string();
                        let version_id = components[3].as_os_str().to_os_string();

                        if components[4].as_os_str() != "extension.jar" {
                            continue;
                        }

                        managed_mods_in_mods_dir.push((path, ty, provider, id, version_id));
                    }
                }
            }
        }
    }

    // remove symlinks that point to extensions that are no longer in the manifest
    for (path, type_, provider, id, version_id) in &managed_mods_in_mods_dir {
        if !manifest.extensions.iter().any(|e| {
            e.type_ == *type_
                && OsStr::new(&e.provider) == provider
                && OsStr::new(&e.id) == id
                && OsStr::new(&e.version_id) == version_id
        }) {
            debug!(
                "Removing symlink at '{}' that points to an extension that is no longer in the manifest",
                path.display()
            );
            tokio::fs::remove_file(path).await?;
        }
    }

    // create symlinks for extensions in the manifest that are not yet symlinked
    for extension in &manifest.extensions {
        if managed_mods_in_mods_dir
            .iter()
            .any(|(_, type_, provider, id, version_id)| {
                *type_ == extension.type_
                    && provider.to_string_lossy() == extension.provider
                    && id.to_string_lossy() == extension.id
                    && version_id.to_string_lossy() == extension.version_id
            })
        {
            continue;
        }

        let provider = get_extension_provider(&extension.provider)
            .with_context(|| format!("Unknown extension provider '{}'", extension.provider))?;

        let target_path = provider
            .get_extension_jar_path(extension.type_, &extension.id, &extension.version_id)
            .await
            .context("Failed to prepare extension jar")?;

        let mut symlink_path = server_dir.to_path_buf();
        match extension.type_ {
            ExtensionType::Mod => symlink_path.push("mods"),
            ExtensionType::Plugin => symlink_path.push("plugins"),
        }
        tokio::fs::create_dir_all(&symlink_path).await?;
        symlink_path.push(format!(
            "{}-{}-{}-{}.jar",
            extension.name, extension.provider, extension.id, extension.version_id
        ));

        debug!(
            "Creating symlink for extension '{}' at '{}'",
            extension.id,
            symlink_path.display()
        );

        tokio::fs::symlink(&target_path, &symlink_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to create symlink for extension '{}' at '{}'",
                    extension.id,
                    symlink_path.display()
                )
            })?;
    }

    Ok(())
}

fn command_substitute_placeholders(
    command: &[OsString],
    java_path: &Path,
    server_jar_path: &Path,
) -> Vec<OsString> {
    command
        .iter()
        .map(|part| {
            part.replace(OsStr::new("${java}"), java_path.as_os_str())
                .replace(OsStr::new("${server_jar}"), server_jar_path.as_os_str())
        })
        .collect()
}

fn start_command_with_pty(
    command_args_str: &[OsString],
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

fn spawn_process_watcher(id: Uuid, mut process: Child) {
    tokio::spawn(async move {
        match process.wait().await {
            Ok(status) => {
                info!("Server process exited with status: {}", status);

                let mut runner = RUNNER.lock().await;
                let server = runner
                    .running_servers
                    .get(&id)
                    .expect("Server should exist");
                let old_status = server.status.get();
                server.status.set(ServerStatus::Stopped);
                let server = runner
                    .running_servers
                    .remove(&id)
                    .expect("Server should exist");
                drop(runner);

                if !status.success()
                    && server.manifest.restart_on_failure
                    && old_status == ServerStatus::Ready
                {
                    info!("Server is configured to restart on failure. Restarting...");
                    if let Err(e) = do_start_server(&server.server_dir, true, false).await {
                        error!(
                            "Failed to restart server at '{}': {:?}",
                            server.server_dir.display(),
                            e
                        );
                    }
                }
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

        if let Err(err) =
            request_server_stop(server.rcon_port.port(), &server.rcon_password, restarting).await
        {
            drop(runner);
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

async fn request_server_stop(
    rcon_port: u16,
    rcon_password: &str,
    restarting: bool,
) -> anyhow::Result<()> {
    let mut count = 0;
    loop {
        let result: anyhow::Result<()> = async {
            let mut client =
                minecraft_rcon::Client::connect((Ipv4Addr::LOCALHOST, rcon_port), rcon_password)
                    .await
                    .context("Failed to connect to RCON")?;

            if restarting {
                client
                    .execute_command("kick @a The server is restarting. Please try connecting again after a while.")
                    .await?;
            }

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

    server
        .status
        .set(ServerStatus::Stopping { restarting: false });

    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(server.pid as i32),
        nix::sys::signal::Signal::SIGKILL,
    )?;
    Ok(())
}
