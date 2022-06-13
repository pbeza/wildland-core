use std::{fmt::Display, rc::Rc};
use wildland_wallet::SigningKeyType;

use crate::crypto::WalletType;
use crate::{CoreXError, CryptoSigningKeypair, FileWallet, ManifestSigningKeypair};

use sha2::{Digest, Sha256};
use wildland_wallet::Wallet;

#[derive(Clone, Copy, Debug)]
pub enum WildlandIdentityType {
    Forest,
    Device,
}

impl From<WildlandIdentityType> for SigningKeyType {
    fn from(identity_type: WildlandIdentityType) -> Self {
        match identity_type {
            WildlandIdentityType::Forest => SigningKeyType::Forest,
            WildlandIdentityType::Device => SigningKeyType::Device,
        }
    }
}

pub trait WildlandIdentityApi: Display {
    fn get_identity_type(&self) -> WildlandIdentityType;
    fn get_public_key(&self) -> Vec<u8>;
    fn get_private_key(&self) -> Vec<u8>;
    fn get_fingerprint(&self) -> Vec<u8>;
    fn get_fingerprint_string(&self) -> String;
    fn save(&self, wallet: WalletType) -> Result<(), CoreXError>;
}

#[derive(Clone)]
pub struct WildlandIdentity {
    identity_type: WildlandIdentityType,
    keypair: Rc<dyn CryptoSigningKeypair>,
}

impl WildlandIdentity {
    pub fn new(identity_type: WildlandIdentityType, keypair: Rc<dyn CryptoSigningKeypair>) -> Self {
        Self {
            identity_type,
            keypair,
        }
    }
}

impl WildlandIdentityApi for WildlandIdentity {
    fn get_public_key(&self) -> Vec<u8> {
        self.keypair.pubkey_as_bytes().into()
    }

    fn get_private_key(&self) -> Vec<u8> {
        self.keypair.seckey_as_bytes().into()
    }

    fn get_fingerprint(&self) -> Vec<u8> {
        let hash = Sha256::digest(&self.get_public_key());

        hash[..16].into()
    }

    fn get_fingerprint_string(&self) -> String {
        hex::encode(self.get_fingerprint())
    }

    fn get_identity_type(&self) -> WildlandIdentityType {
        self.identity_type
    }

    fn save(&self, wallet: WalletType) -> Result<(), CoreXError> {
        match &wallet {
            WalletType::File => {
                let wallet = FileWallet::new().map_err(|e| {
                    CoreXError::IdentityGenerationError(format!(
                        "Could not instantiate Wallet. {}",
                        e
                    ))
                })?;

                let wallet_keypair = ManifestSigningKeypair::from_keys(
                    self.get_identity_type().into(),
                    self.keypair.seckey_as_bytes(),
                    self.keypair.pubkey_as_bytes(),
                );

                wallet
                    .save_signing_secret(wallet_keypair)
                    .map_err(|e| CoreXError::IdentityGenerationError(e.to_string()))?
            }
        }

        Ok(())
    }
}

impl Display for WildlandIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_fingerprint_string(),)
    }
}
