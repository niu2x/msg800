pub type Result<T> = std::result::Result<T, std::io::Error>;

pub mod msg;
pub mod proxy;
pub mod tunel;
