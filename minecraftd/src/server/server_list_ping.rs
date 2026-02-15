use anyhow::Context;
use minecraft_protocol::{
    packet::{Packet, ProtocolBound, ProtocolState, status_response::StatusResponse},
    raw_packet_stream::RawPacketStream,
    varint::VarInt,
};
use tokio::net::{TcpStream, ToSocketAddrs, lookup_host};

pub async fn server_list_ping(addr: impl ToSocketAddrs) -> anyhow::Result<StatusResponse> {
    let addr = lookup_host(addr)
        .await
        .context("Failed to resolve address")?
        .next()
        .context("No addresses found")?;

    let socket = TcpStream::connect(addr).await?;
    let mut raw_packet_stream = RawPacketStream::new(socket);

    raw_packet_stream
        .write_packet(
            &Packet::Handshake {
                protocol_version: VarInt(-1),
                server_address: addr.ip().to_string(),
                server_port: addr.port(),
                intent: ProtocolState::Status,
            }
            .to_raw_packet(),
        )
        .await?;

    raw_packet_stream
        .write_packet(&Packet::StatusRequest {}.to_raw_packet())
        .await?;

    let response_packet = Packet::from_raw_packet(
        ProtocolState::Status,
        ProtocolBound::Clientbound,
        &raw_packet_stream.read_packet().await?,
    )?;

    if let Packet::StatusResponse { json_response } = response_packet {
        Ok(*json_response)
    } else {
        anyhow::bail!("Unexpected packet received");
    }
}
