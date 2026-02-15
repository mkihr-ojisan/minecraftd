use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::{
    raw_packet::RawPacket,
    varint::{AsyncReadVarInt, AsyncWriteVarInt, VarInt, varint_length},
};

pub struct RawPacketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream: S,
}

impl<S> RawPacketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: S) -> Self {
        Self { stream }
    }

    pub async fn read_packet(&mut self) -> Result<RawPacket, std::io::Error> {
        let length = self.stream.read_varint().await?;
        let packet_id = self.stream.read_varint().await?;
        let mut data = vec![0; length.0 as usize - varint_length(packet_id)];
        self.stream.read_exact(&mut data).await?;

        Ok(RawPacket { packet_id, data })
    }

    pub async fn write_packet(&mut self, packet: &RawPacket) -> Result<(), std::io::Error> {
        let length = packet.data.len() + varint_length(packet.packet_id);

        self.stream.write_varint(VarInt(length as i32)).await?;
        self.stream.write_varint(packet.packet_id).await?;
        self.stream.write_all(&packet.data).await?;

        Ok(())
    }

    pub fn into_inner(self) -> S {
        self.stream
    }
}
