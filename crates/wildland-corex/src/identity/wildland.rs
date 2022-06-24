use crate::{CoreXError, ManifestSigningKeypair};
use sha2::{Digest, Sha256};
use std::{fmt::Display, rc::Rc};
use wildland_crypto::identity::SigningKeypair;
use wildland_wallet::{SigningKeyType, Wallet};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WildlandIdentityType {
    Forest,
    Device,
}

// TODO generate below block of code
impl WildlandIdentityType {
    pub fn is_forest(&self) -> bool {
        *self == Self::Forest
    }

    pub fn is_device(&self) -> bool {
        *self == Self::Device
    }

    pub fn is_same(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl From<WildlandIdentityType> for SigningKeyType {
    fn from(identity_type: WildlandIdentityType) -> Self {
        match identity_type {
            WildlandIdentityType::Forest => SigningKeyType::Forest,
            WildlandIdentityType::Device => SigningKeyType::Device,
        }
    }
}

pub trait WildlandIdentityApi: Display + std::fmt::Debug {
    fn get_type(&self) -> WildlandIdentityType;
    fn get_public_key(&self) -> [u8; 32];
    fn get_private_key(&self) -> [u8; 32];
    fn get_fingerprint(&self) -> Vec<u8>;
    fn get_fingerprint_string(&self) -> String;
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn save(&self) -> Result<(), CoreXError>;
}

type IdentityWalletType = Rc<dyn Wallet>;

#[derive(Debug)]
pub struct WildlandIdentity {
    identity_type: WildlandIdentityType,
    keypair: SigningKeypair,
    name: String,
    wallet: IdentityWalletType,
}

impl WildlandIdentity {
    pub fn new(
        identity_type: WildlandIdentityType,
        keypair: SigningKeypair,
        name: String,
        wallet: IdentityWalletType,
    ) -> Self {
        Self {
            identity_type,
            keypair,
            name,
            wallet,
        }
    }
}

impl WildlandIdentityApi for WildlandIdentity {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn get_public_key(&self) -> [u8; 32] {
        self.keypair.public()
    }

    fn get_private_key(&self) -> [u8; 32] {
        self.keypair.secret()
    }

    fn get_fingerprint(&self) -> Vec<u8> {
        let hash = Sha256::digest(&self.get_public_key());

        hash[..16].into()
    }

    fn get_fingerprint_string(&self) -> String {
        hex::encode(self.get_fingerprint())
    }

    fn get_type(&self) -> WildlandIdentityType {
        self.identity_type
    }

    fn save(&self) -> Result<(), CoreXError> {
        let wallet_keypair = ManifestSigningKeypair::from_keypair(
            self.get_type().into(),
            SigningKeypair::try_from_bytes_slices(self.keypair.public(), self.keypair.secret())
                .unwrap(),
        );

        self.wallet
            .save_signing_secret(wallet_keypair)
            .map_err(CoreXError::IdentitySaveError)
    }
}

impl Display for WildlandIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_fingerprint_string(),)
    }
}
