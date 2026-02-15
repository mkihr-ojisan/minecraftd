use std::io::Cursor;

use anyhow::bail;
use status_response::StatusResponse;

use super::{
    raw_packet::RawPacket,
    stream_ext::ReadExt,
    stream_ext::WriteExt,
    text_component::TextComponent,
    varint::ReadVarInt,
    varint::{VarInt, WriteVarInt},
};

pub mod status_response;

#[derive(Debug, Clone)]
pub enum Packet {
    // State: Handshaking
    // Serverbound
    Handshake {
        protocol_version: VarInt,
        server_address: String,
        server_port: u16,
        intent: ProtocolState,
    },

    // State: Status
    // Clientbound
    StatusResponse {
        json_response: Box<StatusResponse>,
    },
    PongResponse {
        timestamp: i64,
    },
    // Serverbound
    StatusRequest,
    PingRequest {
        timestamp: i64,
    },

    // State: Login
    // Clientbound
    Disconnect {
        reason: TextComponent,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolState {
    Handshaking = 0,
    Status = 1,
    Login = 2,
    Transfer = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolBound {
    Clientbound,
    Serverbound,
}

impl Packet {
    pub fn from_raw_packet(
        state: ProtocolState,
        bound: ProtocolBound,
        raw_packet: &RawPacket,
    ) -> anyhow::Result<Packet> {
        let mut cursor = Cursor::new(&raw_packet.data);

        match state {
            ProtocolState::Handshaking => match bound {
                ProtocolBound::Clientbound => {
                    bail!("There are no clientbound packets in the Handshaking state")
                }
                ProtocolBound::Serverbound => match raw_packet.packet_id.0 {
                    0x00 => {
                        let protocol_version = cursor.read_varint()?;
                        let server_address = cursor.read_string()?;
                        let server_port = cursor.read_unsigned_short()?;
                        let next_state = match cursor.read_varint()?.0 {
                            1 => ProtocolState::Status,
                            2 => ProtocolState::Login,
                            3 => ProtocolState::Transfer,
                            _ => bail!("Invalid next state"),
                        };

                        Ok(Packet::Handshake {
                            protocol_version,
                            server_address,
                            server_port,
                            intent: next_state,
                        })
                    }
                    _ => bail!("Unknown packet id: {}", raw_packet.packet_id.0),
                },
            },
            ProtocolState::Status => match bound {
                ProtocolBound::Clientbound => match raw_packet.packet_id.0 {
                    0x00 => {
                        let json_response = cursor.read_string()?;

                        debug!("parsing json_response: {:?}", json_response);

                        Ok(Packet::StatusResponse {
                            json_response: serde_json::from_str(&json_response)?,
                        })
                    }
                    0x01 => {
                        let timestamp = cursor.read_long()?;
                        Ok(Packet::PongResponse { timestamp })
                    }
                    _ => bail!("Unknown packet id: {}", raw_packet.packet_id.0),
                },
                ProtocolBound::Serverbound => match raw_packet.packet_id.0 {
                    0x00 => Ok(Packet::StatusRequest),
                    0x01 => {
                        let timestamp = cursor.read_long()?;
                        Ok(Packet::PingRequest { timestamp })
                    }
                    _ => bail!("Unknown packet id: {}", raw_packet.packet_id.0),
                },
            },
            ProtocolState::Login => match bound {
                ProtocolBound::Clientbound => match raw_packet.packet_id.0 {
                    0x00 => {
                        let reason_str = cursor.read_string()?;
                        let reason: TextComponent = serde_json::from_str(&reason_str)?;

                        Ok(Packet::Disconnect { reason })
                    }
                    _ => bail!("Unknown packet id: {}", raw_packet.packet_id.0),
                },
                ProtocolBound::Serverbound => {
                    bail!("Unknown packet id: {}", raw_packet.packet_id.0);
                }
            },
            ProtocolState::Transfer => unimplemented!(),
        }
    }

    pub fn to_raw_packet(&self) -> RawPacket {
        let mut data = Vec::new();

        match self {
            Packet::Handshake {
                protocol_version,
                server_address,
                server_port,
                intent: next_state,
            } => {
                data.write_varint(*protocol_version).unwrap();
                data.write_string(server_address).unwrap();
                data.write_unsigned_short(*server_port).unwrap();
                data.write_varint(VarInt(*next_state as i32)).unwrap();
            }
            Packet::StatusResponse { json_response } => {
                data.write_string(&serde_json::to_string(json_response).unwrap())
                    .unwrap();
            }
            Packet::PongResponse { timestamp } => {
                data.write_long(*timestamp).unwrap();
            }
            Packet::StatusRequest => {}
            Packet::PingRequest { timestamp } => {
                data.write_long(*timestamp).unwrap();
            }
            Packet::Disconnect { reason } => {
                data.write_string(&serde_json::to_string(reason).unwrap())
                    .unwrap();
            }
        }

        RawPacket {
            packet_id: self.packet_id(),
            data,
        }
    }

    pub fn packet_id(&self) -> VarInt {
        match self {
            Packet::Handshake { .. } => VarInt(0x00),
            Packet::StatusResponse { .. } => VarInt(0x00),
            Packet::PongResponse { .. } => VarInt(0x01),
            Packet::StatusRequest => VarInt(0x00),
            Packet::PingRequest { .. } => VarInt(0x01),
            Packet::Disconnect { .. } => VarInt(0x00),
        }
    }
}
