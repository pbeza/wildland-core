use super::signing_keypair::bytes_key_from_str;
use crate::error::CryptoError;
use crypto_box::{PublicKey as EncryptionPublicKey, SecretKey as EncryptionSecretKey};

#[derive(Debug)]
pub struct EncryptingKeypair {
    pub secret: EncryptionSecretKey,
    pub public: EncryptionPublicKey,
}

impl EncryptingKeypair {
    // TODO unused method
    fn _from_bytes_slices(pubkey: [u8; 32], seckey: [u8; 32]) -> Self {
        Self {
            secret: EncryptionSecretKey::from(seckey),
            public: EncryptionPublicKey::from(pubkey),
        }
    }

    // TODO unused method
    fn _from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
        let pubkey = bytes_key_from_str(public_key)?;
        let seckey = bytes_key_from_str(secret_key)?;
        Ok(Self::_from_bytes_slices(pubkey, seckey))
    }
}
