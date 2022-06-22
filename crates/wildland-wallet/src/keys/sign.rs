use crate::{SigningKeyType, WalletError};
use sha2::{Digest, Sha256};

static EMPTY_KEY: [u8; 32] = [0u8; 32];

#[derive(Debug)]
pub struct ManifestSigningKeypair {
    private_key: [u8; 32],
    public_key: [u8; 32],
    key_type: SigningKeyType,
}

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

    pub fn fingerprint(&self) -> String {
        let hash = Sha256::digest(&self.public_key);

        hex::encode(&hash[..16])
    }

    pub fn can_sign(&self) -> bool {
        !self.private_key.eq(&EMPTY_KEY)
    }

    pub fn sign(&self, _message: &[u8]) -> Result<(), WalletError> {
        todo!()
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.to_vec()
    }

    pub fn get_private_key(&self) -> Vec<u8> {
        self.private_key.to_vec()
    }

    pub fn get_key_type(&self) -> SigningKeyType {
        self.key_type.clone()
    }
}
