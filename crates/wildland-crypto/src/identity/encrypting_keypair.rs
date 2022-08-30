use super::bytes_key_from_str;
use crate::error::CryptoError;
use crypto_box::{PublicKey as EncryptionPublicKey, SecretKey as EncryptionSecretKey};

/// Keypair that can be used for encryption.
/// See crypto-box crate for details.
#[derive(Debug)]
pub struct EncryptingKeypair {
    pub secret: EncryptionSecretKey,
    pub public: EncryptionPublicKey,
}

impl EncryptingKeypair {
    #[tracing::instrument(level = "debug", ret)]
    fn from_bytes_slices(pubkey: [u8; 32], seckey: [u8; 32]) -> Self {
        Self {
            secret: EncryptionSecretKey::from(seckey),
            public: EncryptionPublicKey::from(pubkey),
        }
    }

    #[tracing::instrument(level = "debug", ret)]
    fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
        let pubkey = bytes_key_from_str(public_key)?;
        let seckey = bytes_key_from_str(secret_key)?;
        Ok(Self::from_bytes_slices(pubkey, seckey))
    }

    /// Creates a randomly generated (non-deterministic) encryption keypair.
    /// This keypair can be used as Single-use Transient Encryption Keypair (STEK).
    #[tracing::instrument(level = "debug", ret)]
    pub fn new() -> Self {
        let mut rng = rand_core::OsRng;
        let secret = EncryptionSecretKey::generate(&mut rng);
        let public = secret.public_key();
        Self { secret, public }
    }
}

impl Default for EncryptingKeypair {
    #[tracing::instrument(level = "debug", ret)]
    fn default() -> Self {
        Self::new()
    }
}
