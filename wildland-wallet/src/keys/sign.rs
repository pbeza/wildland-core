use anyhow::Result;
use sha2::{Digest, Sha256};
// use wildland_crypto::identity::keys::SigningKeyPair as CryptoSigningKeyPair;

use crate::{SigningKeyType, WalletKeypair};

static EMPTY_KEY: [u8; 32] = [0u8; 32];

pub struct ManifestSigningKeypair {
    private_key: [u8; 32],
    public_key: [u8; 32],
    key_type: SigningKeyType,
}

// impl<T: CryptoSigningKeyPair> From<T> for ManifestSigningKeypair {
//     fn from(value: T) -> Self {
//         ManifestSigningKeypair::from_keys(value.seckey_as_bytes(), value.pubkey_as_bytes())
//     }
// }

impl ManifestSigningKeypair {
    pub fn from_keys(
        key_type: SigningKeyType,
        private_key: [u8; 32],
        public_key: [u8; 32],
    ) -> Self {
        Self {
            private_key,
            public_key,
            key_type,
        }
    }

    pub fn from_public_key(key_type: SigningKeyType, key: [u8; 32]) -> Self {
        Self {
            private_key: EMPTY_KEY,
            public_key: key,
            key_type,
        }
    }
}

impl WalletKeypair for ManifestSigningKeypair {
    fn fingerprint(&self) -> String {
        let hash = Sha256::digest(&self.public_key);

        hex::encode(&hash[..16])
    }

    fn can_sign(&self) -> bool {
        !self.private_key.eq(&EMPTY_KEY)
    }

    fn sign(&self, _message: &[u8]) -> Result<()> {
        todo!()
    }

    fn get_public_key(&self) -> Vec<u8> {
        self.public_key.to_vec()
    }

    fn get_private_key(&self) -> Vec<u8> {
        self.private_key.to_vec()
    }

    fn get_key_type(&self) -> SigningKeyType {
        self.key_type.clone()
    }
}
