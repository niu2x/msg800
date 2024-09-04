pub mod aes {

    use ::aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};

    pub type Key = [u8; 16];
    pub type IV = [u8; 16];

    type Aes128CbcEnc = cbc::Encryptor<::aes::Aes128>;
    type Aes128CbcDec = cbc::Decryptor<::aes::Aes128>;

    pub struct Aes {
        key: Key,
        iv: IV,
    }

    impl Aes {
        pub fn new(key: Key, iv: IV) -> Self {
            Self { key, iv }
        }
        pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
            let key = &self.key;
            let iv = &self.iv;
            let enc = Aes128CbcEnc::new(key.into(), iv.into());

            let mut buf = vec![0; data.len() + 16];
            let enc_len = enc
                .encrypt_padded_b2b_mut::<Pkcs7>(data, &mut buf)
                .unwrap()
                .len();
            buf.resize(enc_len, 0);
            buf
        }
        pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
            let key = &self.key;
            let iv = &self.iv;
            let enc = Aes128CbcDec::new(key.into(), iv.into());

            let mut buf = vec![0; data.len()];
            let enc_len = enc
                .decrypt_padded_b2b_mut::<Pkcs7>(data, &mut buf)
                .unwrap()
                .len();
            buf.resize(enc_len, 0);
            buf
        }
    }
}
