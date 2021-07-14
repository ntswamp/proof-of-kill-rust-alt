pub mod character;
pub mod fight;
pub mod crypto;
pub mod block;
pub mod blockchain;

pub type Result<T> = std::result::Result<T, std::io::Error>;