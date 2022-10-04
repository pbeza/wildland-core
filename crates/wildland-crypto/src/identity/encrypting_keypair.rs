use super::bytes_key_from_str;
use crate::error::CryptoError;
use crypto_box::{aead::Aead, PublicKey as EncryptionPublicKey, SecretKey as EncryptionSecretKey};
use hex::ToHex;

/// Keypair that can be used for encryption.
/// See crypto-box crate for details.
#[derive(Debug)]
pub struct EncryptingKeypair {
    pub secret: EncryptionSecretKey,
    pub public: EncryptionPublicKey,
}

impl EncryptingKeypair {
    // TODO:WILX-209 unused method
    #[tracing::instrument(level = "debug", ret)]
    fn from_bytes_slices(pubkey: [u8; 32], seckey: [u8; 32]) -> Self {
        Self {
            secret: EncryptionSecretKey::from(seckey),
            public: EncryptionPublicKey::from(pubkey),
        }
    }

    // TODO:WILX-209 unused method
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

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn encode_pub(&self) -> String {
        self.public.as_bytes().encode_hex::<String>()
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn decrypt(&self, cipher_text: Vec<u8>) -> Result<Vec<u8>, CryptoError> {
        let salsa_box = crypto_box::Box::new(&self.public, &self.secret);
        // TODO once must be the same during encryption (evs side) and decryption (corex side)
        // The below one is only a placeholder
        // alternative: choose algorithm not requiring nonce (sealed_box)
        let mut rng = rand_core::OsRng;
        salsa_box
            .decrypt(
                &crypto_box::generate_nonce(&mut rng),
                cipher_text.as_slice(),
            )
            .map_err(|_| CryptoError::DecryptionError)
    }
}

impl Default for EncryptingKeypair {
    #[tracing::instrument(level = "debug", ret)]
    fn default() -> Self {
        Self::new()
    }
}
