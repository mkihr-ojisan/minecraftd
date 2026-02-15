use std::io::{Read, Write};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::{stream_ext::ReadExt as _, stream_ext_async::ReadExt as _};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarInt(pub i32);

pub trait ReadVarInt {
    fn read_varint(&mut self) -> Result<VarInt, std::io::Error>;
}

pub trait WriteVarInt {
    fn write_varint(&mut self, value: VarInt) -> Result<(), std::io::Error>;
}

pub trait AsyncReadVarInt {
    fn read_varint(&mut self) -> impl Future<Output = Result<VarInt, std::io::Error>>;
}

pub trait AsyncWriteVarInt {
    fn write_varint(&mut self, value: VarInt) -> impl Future<Output = Result<(), std::io::Error>>;
}

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

impl<R: Read> ReadVarInt for R {
    fn read_varint(&mut self) -> Result<VarInt, std::io::Error> {
        let mut value = 0i32;
        let mut position = 0;
        let mut current_byte: u8;

        loop {
            current_byte = self.read_unsigned_byte()?;
            value |= ((current_byte & SEGMENT_BITS) as i32) << position;

            if (current_byte & CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "VarInt too long",
                ));
            }
        }

        Ok(VarInt(value))
    }
}

impl<W: Write> WriteVarInt for W {
    fn write_varint(&mut self, value: VarInt) -> Result<(), std::io::Error> {
        let mut value = value.0 as u32;

        loop {
            if (value & !(SEGMENT_BITS as u32)) == 0 {
                self.write_all(&[value as u8])?;
                return Ok(());
            }

            self.write_all(&[((value & (SEGMENT_BITS as u32)) as u8 | CONTINUE_BIT)])?;

            value >>= 7;
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncReadVarInt for R {
    async fn read_varint(&mut self) -> Result<VarInt, std::io::Error> {
        let mut value = 0i32;
        let mut position = 0;
        let mut current_byte: u8;

        loop {
            current_byte = self.read_unsigned_byte().await?;
            value |= ((current_byte & SEGMENT_BITS) as i32) << position;

            if (current_byte & CONTINUE_BIT) == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "VarInt too long",
                ));
            }
        }

        Ok(VarInt(value))
    }
}

impl<W: AsyncWrite + Unpin> AsyncWriteVarInt for W {
    async fn write_varint(&mut self, value: VarInt) -> Result<(), std::io::Error> {
        let mut value = value.0 as u32;

        loop {
            if (value & !(SEGMENT_BITS as u32)) == 0 {
                self.write_all(&[value as u8]).await?;
                return Ok(());
            }

            self.write_all(&[((value & (SEGMENT_BITS as u32)) as u8 | CONTINUE_BIT)])
                .await?;

            value >>= 7;
        }
    }
}

pub fn varint_length(value: VarInt) -> usize {
    let mut value = value.0 as u32;
    let mut length = 1;

    loop {
        if (value & !(SEGMENT_BITS as u32)) == 0 {
            return length;
        }

        length += 1;
        value >>= 7;
    }
}
