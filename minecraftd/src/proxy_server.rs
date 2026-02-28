use std::{
    collections::HashMap,
    net::Ipv4Addr,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use anyhow::{Context, bail};
use minecraft_protocol::{
    packet::{
        Packet, ProtocolBound, ProtocolState,
        status_response::{StatusResponse, Version},
    },
    raw_packet_stream::RawPacketStream,
    text_component::TextComponent,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    signal::unix::{SignalKind, signal},
    sync::Mutex,
};
use uuid::Uuid;

use crate::{
    config::get_config,
    metrics::{self, MetricsCollector, MetricsCollectorContext},
    runner::{self, ServerStatus},
    util::BoxedFuture,
};

static PROXY_SERVER: LazyLock<Mutex<ProxyServer>> =
    LazyLock::new(|| Mutex::new(ProxyServer::default()));

#[derive(Default)]
struct ProxyServer {
    hostname_to_server_id: HashMap<String, Uuid>,
    servers: HashMap<Uuid, Server>,
}

struct Server {
    hostname: String,
    port: u16,
    stats: Arc<Stats>,
}

#[derive(Default)]
struct Stats {
    received_bytes: AtomicU64,
    sent_bytes: AtomicU64,
}

pub async fn init() -> anyhow::Result<()> {
    metrics::register_metrics_collector(ProxyMetricsCollector::default()).await;
    start_server().await?;

    Ok(())
}

pub async fn register_server(server_id: Uuid, hostname: &str, port: u16) {
    let mut proxy_server = PROXY_SERVER.lock().await;
    proxy_server
        .hostname_to_server_id
        .insert(hostname.to_string(), server_id);
    proxy_server.servers.insert(
        server_id,
        Server {
            hostname: hostname.to_string(),
            port,
            stats: Default::default(),
        },
    );
}

pub async fn unregister_server(server_id: Uuid) {
    let mut proxy_server = PROXY_SERVER.lock().await;
    if let Some(server) = proxy_server.servers.remove(&server_id) {
        proxy_server.hostname_to_server_id.remove(&server.hostname);
    }
}

async fn start_server() -> anyhow::Result<()> {
    let bind_address = &get_config().proxy_server.bind_address;

    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .with_context(|| format!("Failed to bind proxy server to address {}", bind_address))?;

    info!(
        "Proxy server listening on {}",
        listener
            .local_addr()
            .context("Failed to get local address")?
    );

    let mut sigint = signal(SignalKind::interrupt()).context("Failed to set up SIGINT handler")?;
    let mut sigterm =
        signal(SignalKind::terminate()).context("Failed to set up SIGTERM handler")?;

    tokio::spawn(async move {
        loop {
            let (socket, addr) = tokio::select! {
                _ = sigint.recv() => {
                    info!("Received SIGINT, shutting down proxy server");
                    break;
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, shutting down proxy server");
                    break;
                }
                result = listener.accept() => match result {
                    Ok(conn) => conn,
                    Err(e) => {
                        error!("Failed to accept incoming connection: {:?}", e);
                        continue;
                    }
                },
            };

            info!("Accepted connection from {}", addr);

            tokio::spawn(async move {
                if let Err(e) = handle_client(socket).await {
                    error!("Error handling client {}: {:?}", addr, e);
                }
            });
        }
    });

    Ok(())
}

async fn handle_client(socket: TcpStream) -> anyhow::Result<()> {
    let peer_addr = socket.peer_addr().context("Failed to get client address")?;

    socket
        .set_nodelay(true)
        .context("Failed to set TCP_NODELAY")?;

    let mut raw_packet_stream = RawPacketStream::new(socket);

    let raw_handshake_packet = raw_packet_stream
        .read_packet()
        .await
        .context("Failed to read handshake packet")?;
    let handshake_packet = Packet::from_raw_packet(
        ProtocolState::Handshaking,
        ProtocolBound::Serverbound,
        &raw_handshake_packet,
    )
    .context("Failed to read handshake packet")?;

    let Packet::Handshake {
        server_address,
        intent,
        ..
    } = handshake_packet
    else {
        bail!("Expected handshake packet");
    };

    debug!(
        "Received handshake packet from client {peer_addr} with server address '{server_address}' and intent '{intent:?}'"
    );

    macro_rules! send_error_message {
        ($message:expr) => {
            if intent != ProtocolState::Status {
                raw_packet_stream
                    .write_packet(
                        &Packet::Disconnect {
                            reason: $message,
                        }
                        .to_raw_packet(),
                    )
                    .await
                    .context("Failed to send disconnect packet")?;
                info!("Disconnected: {}", $message);
                return Ok(());
            } else {
                debug!("Entering fallback server list ping handler for client {peer_addr} with error message: {}", $message);
                fallback_server_list_ping_server(&mut raw_packet_stream, $message).await?;
                return Ok(());
            }
        };
    }

    let proxy_server = PROXY_SERVER.lock().await;
    let Some(&server_id) = proxy_server.hostname_to_server_id.get(&server_address) else {
        send_error_message!(get_config().messages.server_not_found.clone());
    };

    let server = proxy_server
        .servers
        .get(&server_id)
        .expect("Server ID was found in hostname_to_server_id but not in servers map");
    let server_port = server.port;
    let stats = server.stats.clone();
    drop(proxy_server);

    match runner::get_server_status(server_id).await {
        Some(ServerStatus::Starting { restarting: false }) => {
            send_error_message!(get_config().messages.server_starting.clone());
        }
        Some(ServerStatus::Ready) => {
            // proceed to connect
        }
        Some(ServerStatus::Stopping { restarting: false }) => {
            send_error_message!(get_config().messages.server_stopping.clone());
        }
        Some(
            ServerStatus::Starting { restarting: true }
            | ServerStatus::Stopping { restarting: true },
        ) => {
            send_error_message!(get_config().messages.server_restarting.clone());
        }
        Some(ServerStatus::Stopped) => unreachable!(),
        None => {
            send_error_message!(get_config().messages.server_not_found.clone());
        }
    }

    info!("Forwarding client {peer_addr} to server with ID {server_id}",);

    let mut server_socket = TcpStream::connect((Ipv4Addr::LOCALHOST, server_port))
        .await
        .context("Failed to connect to backend server")?;
    server_socket
        .set_nodelay(true)
        .context("Failed to set TCP_NODELAY on backend server connection")?;

    RawPacketStream::new(&mut server_socket)
        .write_packet(&raw_handshake_packet)
        .await
        .context("Failed to forward handshake packet to backend server")?;

    let mut socket = raw_packet_stream.into_inner();

    let result: anyhow::Result<()> = async {
        loop {
            let mut buf1 = [0u8; 1024];
            let mut buf2 = [0u8; 1024];

            tokio::select! {
                n = socket.read(&mut buf1) => {
                    let n = n.context("Failed to read from client socket")?;
                    if n == 0 {
                        break;
                    }
                    stats.received_bytes.fetch_add(n as u64, Ordering::SeqCst);
                    server_socket.write_all(&buf1[..n]).await.context("Failed to write to server socket")?;
                }
                n = server_socket.read(&mut buf2) => {
                    let n = n.context("Failed to read from server socket")?;
                    if n == 0 {
                        break;
                    }
                    stats.sent_bytes.fetch_add(n as u64, Ordering::SeqCst);
                    socket.write_all(&buf2[..n]).await.context("Failed to write to client socket")?;
                }
            }
        }
        Ok(())
    }.await;

    if result.is_ok() {
        debug!("Client {peer_addr} disconnected");
    }

    result
}

async fn fallback_server_list_ping_server<S: AsyncRead + AsyncWrite + Unpin>(
    socket: &mut RawPacketStream<S>,
    error_message: TextComponent,
) -> anyhow::Result<()> {
    loop {
        let packet = match socket.read_packet().await {
            Ok(packet) => packet,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!("Client closed connection during fallback server list ping handler");
                return Ok(());
            }
            Err(e) => bail!("Failed to read packet from client: {:?}", e),
        };
        let packet =
            Packet::from_raw_packet(ProtocolState::Status, ProtocolBound::Serverbound, &packet)
                .context("Failed to parse packet from client")?;

        match packet {
            Packet::StatusRequest => {
                trace!("Received status request packet from client, sending error response");

                let response = Packet::StatusResponse {
                    json_response: Box::new(StatusResponse {
                        version: Version {
                            name: TextComponent::String("".to_string()),
                            protocol: 0,
                        },
                        players: None,
                        description: Some(error_message.clone()),
                        favicon: None,
                        modinfo: None,
                        forge_data: None,
                    }),
                };
                socket
                    .write_packet(&response.to_raw_packet())
                    .await
                    .context("Failed to write status response packet to client")?;
            }
            Packet::PingRequest { timestamp } => {
                trace!("Received ping request packet from client, sending pong response");

                let response = Packet::PongResponse { timestamp };
                socket
                    .write_packet(&response.to_raw_packet())
                    .await
                    .context("Failed to write pong response packet to client")?;
                break;
            }
            _ => {
                bail!("Unexpected packet: {:?}", packet);
            }
        }
    }

    Ok(())
}

pub struct ProxyStats {
    pub received_bytes: u64,
    pub sent_bytes: u64,
}

async fn get_stats() -> HashMap<Uuid, ProxyStats> {
    let proxy_server = PROXY_SERVER.lock().await;
    proxy_server
        .servers
        .iter()
        .map(|(server_id, server)| {
            (
                *server_id,
                ProxyStats {
                    received_bytes: server.stats.received_bytes.load(Ordering::SeqCst),
                    sent_bytes: server.stats.sent_bytes.load(Ordering::SeqCst),
                },
            )
        })
        .collect()
}

#[derive(Default)]
struct ProxyMetricsCollector {
    last_timestamp: Option<Instant>,
    last_stats: HashMap<Uuid, ProxyStats>,
}

impl MetricsCollector for ProxyMetricsCollector {
    fn name(&self) -> &'static str {
        "proxy_metrics_collector"
    }

    fn collect<'a>(
        &'a mut self,
        ctx: &'a mut MetricsCollectorContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let current_values = get_stats().await;
            let now = Instant::now();

            let Some(last_timestamp) = self.last_timestamp else {
                self.last_timestamp = Some(now);
                self.last_stats = current_values;
                return Ok(());
            };

            let elapsed_seconds = now.duration_since(last_timestamp).as_secs_f64();
            if elapsed_seconds <= 0.0 {
                self.last_timestamp = Some(now);
                self.last_stats = current_values;
                return Ok(());
            }

            let timestamp = std::time::SystemTime::now();

            for (server_id, stats) in &current_values {
                let Some(last_stats) = self.last_stats.get(server_id) else {
                    continue;
                };

                let delta_received = stats.received_bytes.wrapping_sub(last_stats.received_bytes);
                let delta_sent = stats.sent_bytes.wrapping_sub(last_stats.sent_bytes);

                let received_rate = delta_received as f64 / elapsed_seconds;
                let sent_rate = delta_sent as f64 / elapsed_seconds;

                ctx.push_metric(
                    *server_id,
                    "proxy_received_bytes_per_second",
                    timestamp,
                    received_rate,
                );
                ctx.push_metric(
                    *server_id,
                    "proxy_sent_bytes_per_second",
                    timestamp,
                    sent_rate,
                );
            }

            self.last_timestamp = Some(now);
            self.last_stats = current_values;
            Ok(())
        })
    }
}
