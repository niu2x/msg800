use crate::crypto::aes::Aes;
use bytebuffer::ByteBuffer;
use rand::{self, Rng};
use tokio::io::{self, AsyncReadExt};

pub struct Message {
    buf: ByteBuffer,
    ciper: Aes,
}

// pub type Key = [u8; 16];
// pub type IV = [u8; 16];

impl Message {
    const MAGIC: u64 = 0x20240828;

    pub fn new(key: &[u8; 16], iv: &[u8; 16]) -> Message {
        Message {
            buf: ByteBuffer::new(),
            ciper: Aes::new(key.clone(), iv.clone()),
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

    pub fn pack(&mut self) -> Vec<u8> {
        let mut block = ByteBuffer::new();
        let body = self.buf.as_bytes();
        let body = self.ciper.encrypt(&body);
        let body_len = body.len() as u64;

        let noise = get_noise();
        let noise_len = noise.len() as u64;

        let total_len = noise_len + body_len + 16;

        let encrypt_total_len = total_len ^ Self::MAGIC;
        let encrypt_noise_len = noise_len ^ Self::MAGIC;

        block.write_u64(encrypt_total_len);
        block.write_u64(encrypt_noise_len);
        block.write_bytes(&noise);
        block.write_bytes(&body);
        block.into_vec()
    }

    pub async fn unpack<T>(&mut self, reader: &mut T) -> io::Result<()>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        let encrypt_total_len = reader.read_u64().await?;
        let total_len = (encrypt_total_len ^ Self::MAGIC) as usize;

        let encrypt_noise_len = reader.read_u64().await?;
        let noise_len = (encrypt_noise_len ^ Self::MAGIC) as usize;

        let mut temp = vec![0; total_len - 16];
        reader.read_exact(&mut temp).await?;
        let buf = self.ciper.decrypt(&temp[noise_len..]);
        self.buf.write_bytes(&buf);
        Ok(())
    }
}

fn get_noise() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let l = rng.gen_range(100..1000);
    let mut noise = vec![0; l];
    rng.fill(&mut noise[..]);
    noise
}
