use sha2::{Digest, Sha256};
use std::fmt::Display;
use wildland_crypto::identity::SigningKeypair;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WildlandIdentityType {
    Forest,
    Device,
}

// TODO WILX-95 generate code for handling enums with binding_wrapper macro
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

#[derive(Debug)]
pub struct WildlandIdentity {
    identity_type: WildlandIdentityType,
    keypair: SigningKeypair,
    name: String,
}

impl WildlandIdentity {
    pub fn new(identity_type: WildlandIdentityType, keypair: SigningKeypair, name: String) -> Self {
        Self {
            identity_type,
            keypair,
            name,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.keypair.public().into()
    }

    pub fn get_private_key(&self) -> Vec<u8> {
        self.keypair.secret().into()
    }

    pub fn get_fingerprint(&self) -> Vec<u8> {
        let hash = Sha256::digest(&self.get_public_key());

        hash[..16].into()
    }

    pub fn get_fingerprint_string(&self) -> String {
        hex::encode(self.get_fingerprint())
    }

    pub fn get_type(&self) -> WildlandIdentityType {
        self.identity_type
    }
}

impl Display for WildlandIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_fingerprint_string(),)
    }
}
