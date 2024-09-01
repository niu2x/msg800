pub type Result<T> = std::result::Result<T, std::io::Error>;

pub mod crypto;
pub mod msg;
pub mod socks5;
pub mod tunel;
