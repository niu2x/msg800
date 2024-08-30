use bytebuffer::ByteBuffer;
use tokio::io::{self, AsyncReadExt};

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::{self, Rng};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

pub struct Message {
    buf: ByteBuffer,
    cipher_enc: Aes128CbcEnc,
    cipher_dec: Aes128CbcDec,
}

// pub type Key = [u8; 16];
// pub type IV = [u8; 16];

impl Message {
    const MAGIC: u64 = 0x20240828;

    pub fn new(key: &[u8; 16], iv: &[u8; 16]) -> Message {
        Message {
            buf: ByteBuffer::new(),
            cipher_enc: Aes128CbcEnc::new(key.into(), iv.into()),
            cipher_dec: Aes128CbcDec::new(key.into(), iv.into()),
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
        let mut buf = vec![0; body.len() + 16];
        let body = self
            .cipher_enc
            .clone()
            .encrypt_padded_b2b_mut::<Pkcs7>(&body, &mut buf)
            .unwrap();
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
        let buf = self
            .cipher_dec
            .clone()
            .decrypt_padded_mut::<Pkcs7>(&mut temp[noise_len..])
            .unwrap();
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
