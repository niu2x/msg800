use bytebuffer::ByteBuffer;
use std::error::Error;
use tokio::io::AsyncReadExt;

pub struct Message {
    buf: ByteBuffer,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const MAGIC: u64 = 0x20240828;

impl Message {
    pub fn new() -> Message {
        Message {
            buf: ByteBuffer::new(),
        }
    }

    pub fn write_u8(&mut self, x: u8) {
        self.buf.write_u8(x);
    }
    pub fn write_u16(&mut self, x: u16) {
        self.buf.write_u16(x);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buf.write_bytes(bytes);
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.buf.as_bytes()
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut block = ByteBuffer::new();
        let body = self.buf.as_bytes();
        let body_len = body.len();
        let total_len: u64 = (body_len + 8) as u64;
        let encrypt_total_len = total_len ^ MAGIC;
        block.write_u64(encrypt_total_len);
        block.write_bytes(&body);
        block.into_vec()
    }

    pub async fn unpack<T>(&mut self, reader: &mut T) -> Result<()>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        let encrypt_total_len = reader.read_u64().await?;
        let total_len = (encrypt_total_len ^ MAGIC) as usize;
        let mut temp = vec![0; total_len];
        reader.read_exact(&mut temp).await?;
        self.buf.write_bytes(&temp);
        Ok(())
    }
}
