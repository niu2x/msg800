use std::error::Error;
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub mod msg;
pub mod proxy;
pub mod tunel;
