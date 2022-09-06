mod error;
mod identity;
mod lss;
mod user;

pub use error::*;
pub use identity::{master::*, wildland::*};
pub use lss::*;
pub use user::{generate_random_mnemonic, CreateUserInput, UserService};
pub use wildland_crypto::{error::CryptoError, identity::MnemonicPhrase};
pub use wildland_local_secure_storage::{FileLSS, LocalSecureStorage, LssError};

pub type CorexResult<T> = Result<T, CoreXError>;

#[cfg(test)]
pub mod test_utilities {
    use crate::WildlandIdentity;
    use wildland_crypto::identity::SigningKeypair;

    pub static SIGNING_PUBLIC_KEY: &str =
        "1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f";
    pub static SIGNING_SECRET_KEY: &str =
        "e02cdfa23ad7d94508108ad41410e556c5b0737e9c264d4a2304a7a45894fc57";

    pub fn create_signing_keypair() -> SigningKeypair {
        SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap()
    }

    pub fn create_wildland_forest_identity() -> WildlandIdentity {
        WildlandIdentity::Forest(0, create_signing_keypair())
    }
}
