use crate::{CoreXError, CryptoSigningKeypair, ManifestSigningKeypair, WalletFactory};
use sha2::{Digest, Sha256};
use std::{fmt::Display, rc::Rc};
use wildland_wallet::SigningKeyType;

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
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn save(&self) -> Result<(), CoreXError>;
}

#[derive(Clone)]
pub struct WildlandIdentity<W: WalletFactory> {
    identity_type: WildlandIdentityType,
    keypair: Rc<dyn CryptoSigningKeypair>,
    name: String,
    wallet: W,
}

impl<W: WalletFactory> WildlandIdentity<W> {
    pub fn new(
        identity_type: WildlandIdentityType,
        keypair: Rc<dyn CryptoSigningKeypair>,
        name: String,
        wallet: W,
    ) -> Self {
        Self {
            identity_type,
            keypair,
            name,
            wallet,
        }
    }
}

impl<W: WalletFactory> WildlandIdentityApi for WildlandIdentity<W> {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

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

    fn save(&self) -> Result<(), CoreXError> {
        let wallet_keypair = ManifestSigningKeypair::from_keys(
            self.get_identity_type().into(),
            self.keypair.seckey_as_bytes(),
            self.keypair.pubkey_as_bytes(),
        );

        self.wallet
            .save_signing_secret(wallet_keypair)
            .map_err(|e| CoreXError::IdentityGenerationError(e.to_string()))?;

        Ok(())
    }
}

impl<W: WalletFactory> Display for WildlandIdentity<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_fingerprint_string(),)
    }
}
