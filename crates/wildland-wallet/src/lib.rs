pub mod api;
pub mod keys;
pub mod wallet;

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub use api::*;
pub use keys::sign::ManifestSigningKeypair;
pub use wallet::{
    file::{file_wallet_factory, FileWallet},
    WalletFactoryType,
};

#[derive(Debug)]
pub enum WalletError {
    TODOError(String),
}
