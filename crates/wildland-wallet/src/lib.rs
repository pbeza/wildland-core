pub mod api;
pub mod keys;
pub mod wallet;

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub use api::*;
pub use keys::sign::ManifestSigningKeypair;
use thiserror::Error;
use wildland_crypto::error::CryptoError;

#[derive(Debug, PartialEq, Error, Eq, Clone)]
pub enum WalletError {
    #[error("File error: {0}")]
    FileError(String),
    #[error("Cryptographic key error: {0}")]
    KeyError(String),
    #[error("Crypto error: {0}")]
    Crypto(CryptoError),
}
