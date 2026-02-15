use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::varint::{AsyncReadVarInt, AsyncWriteVarInt, VarInt};

pub trait ReadExt {
    fn read_boolean(&mut self) -> impl Future<Output = Result<bool, std::io::Error>>;
    fn read_byte(&mut self) -> impl Future<Output = Result<i8, std::io::Error>>;
    fn read_unsigned_byte(&mut self) -> impl Future<Output = Result<u8, std::io::Error>>;
    fn read_short(&mut self) -> impl Future<Output = Result<i16, std::io::Error>>;
    fn read_unsigned_short(&mut self) -> impl Future<Output = Result<u16, std::io::Error>>;
    fn read_int(&mut self) -> impl Future<Output = Result<i32, std::io::Error>>;
    fn read_long(&mut self) -> impl Future<Output = Result<i64, std::io::Error>>;
    fn read_float(&mut self) -> impl Future<Output = Result<f32, std::io::Error>>;
    fn read_double(&mut self) -> impl Future<Output = Result<f64, std::io::Error>>;
    fn read_string(&mut self) -> impl Future<Output = Result<String, std::io::Error>>;
}

impl<R: AsyncRead + Unpin> ReadExt for R {
    async fn read_boolean(&mut self) -> Result<bool, std::io::Error> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).await?;
        Ok(buf[0] != 0)
    }

    async fn read_byte(&mut self) -> Result<i8, std::io::Error> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).await?;
        Ok(i8::from_be_bytes(buf))
    }

    async fn read_unsigned_byte(&mut self) -> Result<u8, std::io::Error> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).await?;
        Ok(u8::from_be_bytes(buf))
    }

    async fn read_short(&mut self) -> Result<i16, std::io::Error> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf).await?;
        Ok(i16::from_be_bytes(buf))
    }

    async fn read_unsigned_short(&mut self) -> Result<u16, std::io::Error> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf).await?;
        Ok(u16::from_be_bytes(buf))
    }

    async fn read_int(&mut self) -> Result<i32, std::io::Error> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf).await?;
        Ok(i32::from_be_bytes(buf))
    }

    async fn read_long(&mut self) -> Result<i64, std::io::Error> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf).await?;
        Ok(i64::from_be_bytes(buf))
    }

    async fn read_float(&mut self) -> Result<f32, std::io::Error> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf).await?;
        Ok(f32::from_be_bytes(buf))
    }

    async fn read_double(&mut self) -> Result<f64, std::io::Error> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf).await?;
        Ok(f64::from_be_bytes(buf))
    }

    async fn read_string(&mut self) -> Result<String, std::io::Error> {
        let length = self.read_varint().await?;
        let mut buf = vec![0u8; length.0 as usize];
        self.read_exact(&mut buf).await?;
        String::from_utf8(buf).or(Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid UTF-8",
        )))
    }
}

pub trait WriteExt {
    fn write_boolean(&mut self, value: bool) -> impl Future<Output = Result<(), std::io::Error>>;
    fn write_byte(&mut self, value: i8) -> impl Future<Output = Result<(), std::io::Error>>;
    fn write_unsigned_byte(
        &mut self,
        value: u8,
    ) -> impl Future<Output = Result<(), std::io::Error>>;
    fn write_short(&mut self, value: i16) -> impl Future<Output = Result<(), std::io::Error>>;
    fn write_unsigned_short(
        &mut self,
        value: u16,
    ) -> impl Future<Output = Result<(), std::io::Error>>;
    fn write_int(&mut self, value: i32) -> impl Future<Output = Result<(), std::io::Error>>;
    fn write_long(&mut self, value: i64) -> impl Future<Output = Result<(), std::io::Error>>;
    fn write_float(&mut self, value: f32) -> impl Future<Output = Result<(), std::io::Error>>;
    fn write_double(&mut self, value: f64) -> impl Future<Output = Result<(), std::io::Error>>;
    fn write_string(&mut self, value: &str) -> impl Future<Output = Result<(), std::io::Error>>;
}

impl<W: AsyncWrite + Unpin> WriteExt for W {
    async fn write_boolean(&mut self, value: bool) -> Result<(), std::io::Error> {
        self.write_all(&[value as u8]).await?;
        Ok(())
    }

    async fn write_byte(&mut self, value: i8) -> Result<(), std::io::Error> {
        self.write_all(&value.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_unsigned_byte(&mut self, value: u8) -> Result<(), std::io::Error> {
        self.write_all(&value.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_short(&mut self, value: i16) -> Result<(), std::io::Error> {
        self.write_all(&value.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_unsigned_short(&mut self, value: u16) -> Result<(), std::io::Error> {
        self.write_all(&value.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_int(&mut self, value: i32) -> Result<(), std::io::Error> {
        self.write_all(&value.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_long(&mut self, value: i64) -> Result<(), std::io::Error> {
        self.write_all(&value.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_float(&mut self, value: f32) -> Result<(), std::io::Error> {
        self.write_all(&value.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_double(&mut self, value: f64) -> Result<(), std::io::Error> {
        self.write_all(&value.to_be_bytes()).await?;
        Ok(())
    }

    async fn write_string(&mut self, value: &str) -> Result<(), std::io::Error> {
        let length = value.len() as i32;
        self.write_varint(VarInt(length)).await?;
        self.write_all(value.as_bytes()).await?;
        Ok(())
    }
}
