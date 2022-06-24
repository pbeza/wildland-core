use crate::{SigningKeyType, WalletError};
use sha2::{Digest, Sha256};
use wildland_crypto::identity::SigningKeypair;

#[derive(Debug)]
pub struct ManifestSigningKeypair {
    keypair: SigningKeypair,
    key_type: SigningKeyType,
}

impl ManifestSigningKeypair {
    pub fn from_keypair(key_type: SigningKeyType, keypair: SigningKeypair) -> Self {
        Self { keypair, key_type }
    }

    pub fn fingerprint(&self) -> String {
        let hash = Sha256::digest(&self.keypair.public());

        hex::encode(&hash[..16])
    }

    pub fn sign(&self, _message: &[u8]) -> Result<(), WalletError> {
        todo!()
    }

    pub fn get_public_key(&self) -> [u8; 32] {
        self.keypair.public()
    }

    pub fn get_private_key(&self) -> [u8; 32] {
        self.keypair.secret()
    }

    pub fn get_key_type(&self) -> SigningKeyType {
        self.key_type.clone()
    }
}
