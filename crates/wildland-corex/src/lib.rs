mod error;
mod identity;
mod wallet;
mod user;

pub use error::*;
pub use identity::{master::*, wildland::*};
pub use wallet::create_file_wallet;
pub use user::{create_user,WildlandUser, CreateUserPayload};
pub use wildland_crypto::identity::{MnemonicPhrase, MNEMONIC_PHRASE_LEN};
pub use wildland_wallet::{ManifestSigningKeypair, Wallet, WalletError};

type CoreXResult<T> = Result<T, CoreXError>;

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
