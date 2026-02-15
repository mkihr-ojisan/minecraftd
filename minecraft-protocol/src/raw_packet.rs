use super::varint::VarInt;

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub packet_id: VarInt,
    pub data: Vec<u8>,
}
