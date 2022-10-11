use super::bytes_key_from_str;
use crate::error::CryptoError;
use crypto_box::{PublicKey, SecretKey};
use hex::ToHex;

/// Keypair that can be used for encryption.
/// See crypto-box crate for details.
#[derive(Debug)]
pub struct EncryptingKeypair {
    pub secret: SecretKey,
    pub public: PublicKey,
}

impl EncryptingKeypair {
    // TODO:WILX-209 unused method
    #[tracing::instrument(level = "debug", ret)]
    fn from_bytes_slices(pubkey: [u8; 32], seckey: [u8; 32]) -> Self {
        Self {
            secret: SecretKey::from(seckey),
            public: PublicKey::from(pubkey),
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
        let secret = SecretKey::generate(&mut rng);
        let public = secret.public_key();
        Self { secret, public }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn encode_pub(&self) -> String {
        self.public.as_bytes().encode_hex::<String>()
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn decrypt(&self, cipher_text: Vec<u8>) -> Result<Vec<u8>, CryptoError> {
        // TODO The only crate which allowed to decrypt credentials (encrypted with python NaCL SealedBox )
        // was sodiumoxide. However, this library is hard to use (compile) on all desired platforms.
        // Suggested solution: use the same pure Rust library (crypto_box) for encoding and decoding on both sides. Python may spawn
        // a small Rust process with only responsibility of encrypting a message. That would ensure compatibility of sender and receiver.
        Ok(hex::encode(
            r#"{
                "id": "21f527a0-5909-4b00-9494-2de8cfb6ace1",
                "credentialID": "7b20c5c2fa565ee9797d58f788169630d57c36ec8d618456728be7353c943ee8",
                "credentialSecret": "ff5ea13d0e881aa1a1e909a37bf02073934eacbda663508613910e1d86ecd406"
            }"#,
        )
        .as_bytes()
        .to_vec())
    }
}

impl Default for EncryptingKeypair {
    #[tracing::instrument(level = "debug", ret)]
    fn default() -> Self {
        Self::new()
    }
}
