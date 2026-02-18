use std::net::Ipv4Addr;

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
};

use crate::{
    config::get_config,
    server::runner::{self, ServerStatus},
};

pub async fn start() -> anyhow::Result<()> {
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
                            reason: TextComponent::String($message),
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

    let Some(server_id) = runner::get_server_id_by_hostname(&server_address).await else {
        send_error_message!("Server is not running or does not exist".to_string());
    };

    match runner::get_server_status(server_id).await {
        Some(ServerStatus::Starting { restarting: false }) => {
            send_error_message!("Server is starting up, please try again later".to_string());
        }
        Some(ServerStatus::Ready) => {
            // proceed to connect
        }
        Some(ServerStatus::Stopping { restarting: false }) => {
            send_error_message!("Server is stopping.".to_string());
        }
        Some(
            ServerStatus::Starting { restarting: true }
            | ServerStatus::Stopping { restarting: true },
        ) => {
            send_error_message!("Server is restarting, please try again later".to_string());
        }
        Some(ServerStatus::Stopped) => unreachable!(),
        None => {
            send_error_message!("Server is not running or does not exist".to_string());
        }
    }

    info!("Forwarding client {peer_addr} to server with ID {server_id}",);

    let Some(server_port) = runner::get_server_port(server_id).await else {
        send_error_message!("Server is not running or does not exist".to_string());
    };

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
    loop {
        let mut buf1 = [0u8; 1024];
        let mut buf2 = [0u8; 1024];

        tokio::select! {
            n = socket.read(&mut buf1) => {
                let n = n.context("Failed to read from client socket")?;
                if n == 0 {
                    break;
                }
                server_socket.write_all(&buf1[..n]).await.context("Failed to write to server socket")?;
            }
            n = server_socket.read(&mut buf2) => {
                let n = n.context("Failed to read from server socket")?;
                if n == 0 {
                    break;
                }
                socket.write_all(&buf2[..n]).await.context("Failed to write to client socket")?;
            }
        }
    }

    info!("Client {peer_addr} disconnected");

    Ok(())
}

async fn fallback_server_list_ping_server<S: AsyncRead + AsyncWrite + Unpin>(
    socket: &mut RawPacketStream<S>,
    error_message: String,
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
                        description: Some(TextComponent::String(error_message.clone())),
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
