use std::{borrow::Cow, io::Cursor};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

#[macro_use]
extern crate log;

const SERVERDATA_AUTH: i32 = 3;
const SERVERDATA_AUTH_RESPONSE: i32 = 2;
const SERVERDATA_EXECCOMMAND: i32 = 2;
const SERVERDATA_RESPONSE_VALUE: i32 = 0;

pub struct Client {
    stream: TcpStream,
}

struct Packet<'a> {
    id: i32,
    type_: i32,
    body: Cow<'a, [u8]>,
}

impl Client {
    pub async fn connect(address: impl ToSocketAddrs, password: &str) -> tokio::io::Result<Self> {
        let stream = TcpStream::connect(address).await?;
        let mut socket = Client { stream };

        socket
            .send_packet(&Packet {
                id: 1,
                type_: SERVERDATA_AUTH,
                body: Cow::Borrowed(password.as_bytes()),
            })
            .await?;

        let response = socket.receive_packet().await?;
        if response.type_ != SERVERDATA_AUTH_RESPONSE || response.id == -1 {
            return Err(tokio::io::Error::other("Authentication failed"));
        }

        Ok(socket)
    }

    async fn send_packet(&mut self, packet: &Packet<'_>) -> tokio::io::Result<()> {
        trace!(
            "Sending packet: id={}, type={}, body={}",
            packet.id,
            packet.type_,
            String::from_utf8_lossy(&packet.body)
        );

        let mut packet_bytes = Cursor::new(Vec::<u8>::with_capacity(packet.body.len() + 10));

        let length = (packet.body.len() + 10) as i32;
        packet_bytes.write_all(&length.to_le_bytes()).await?;
        packet_bytes.write_all(&packet.id.to_le_bytes()).await?;
        packet_bytes.write_all(&packet.type_.to_le_bytes()).await?;
        packet_bytes.write_all(&packet.body).await?;
        packet_bytes.write_all(&[0, 0]).await?;

        self.stream.write_all(&packet_bytes.into_inner()).await?;
        self.stream.flush().await?;

        Ok(())
    }

    async fn receive_packet(&mut self) -> tokio::io::Result<Packet<'static>> {
        let mut length_buf = [0u8; 4];
        self.stream.read_exact(&mut length_buf).await?;
        let length = i32::from_le_bytes(length_buf);

        let mut id_buf = [0u8; 4];
        self.stream.read_exact(&mut id_buf).await?;
        let id = i32::from_le_bytes(id_buf);

        let mut type_buf = [0u8; 4];
        self.stream.read_exact(&mut type_buf).await?;
        let type_ = i32::from_le_bytes(type_buf);

        let body_length = length - 10;
        if body_length < 0 {
            return Err(tokio::io::Error::other("Invalid packet length"));
        }

        let mut body = vec![0u8; body_length as usize];
        self.stream.read_exact(&mut body).await?;

        let mut null_bytes = [0u8; 2];
        self.stream.read_exact(&mut null_bytes).await?;

        if null_bytes != [0, 0] {
            return Err(tokio::io::Error::other("Invalid packet termination"));
        }

        trace!(
            "Received packet: id={id}, type={type_}, body={}",
            String::from_utf8_lossy(&body)
        );

        Ok(Packet {
            id,
            type_,
            body: Cow::Owned(body),
        })
    }

    pub async fn execute_command(&mut self, command: &str) -> tokio::io::Result<()> {
        self.send_packet(&Packet {
            id: 2,
            type_: SERVERDATA_EXECCOMMAND,
            body: Cow::Borrowed(command.as_bytes()),
        })
        .await?;

        let response = self.receive_packet().await?;
        if response.type_ != SERVERDATA_RESPONSE_VALUE {
            return Err(tokio::io::Error::other("Invalid response type"));
        }

        Ok(())
    }
}
