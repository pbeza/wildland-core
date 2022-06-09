mod crypto;
mod error;
mod identity;

pub use crypto::*;
pub use error::*;
pub use identity::master::*;
pub use identity::wildland::*;
pub use wildland_crypto::identity::{
    keys::KeyPair as CryptoKeypair, keys::SigningKeyPair as CryptoSigningKeypair, Identity,
    SeedPhraseWords, SEED_PHRASE_LEN,
};
pub use wildland_wallet::{
    FileWallet, ManifestSigningKeypair, SigningKeyType, Wallet, WalletKeypair,
};

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_version_verbose() -> Vec<(&'static str, &'static str)> {
    vec![
        ("CoreX", env!("CARGO_PKG_VERSION")),
        ("CatLib", wildland_catlib::get_version()),
        ("Wallet", wildland_wallet::get_version()),
        ("DFS", wildland_dfs::get_version()),
    ]
}
