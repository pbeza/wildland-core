use crate::keys::sign::ManifestSigningKeypair;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SigningKeyType {
    Forest,
    Device,
}

pub trait WalletKeypair {
    fn can_sign(&self) -> bool;
    fn sign(&self, message: &[u8]) -> Result<()>;
    fn fingerprint(&self) -> String;
    fn get_public_key(&self) -> Vec<u8>;
    fn get_private_key(&self) -> Vec<u8>;
    fn get_key_type(&self) -> SigningKeyType;
}

pub trait WalletFactory: Wallet<ManifestSigningKeypair> + Clone {
    fn new() -> Result<Self>
    where
        Self: Sized;
}

pub trait Wallet<T: WalletKeypair> {
    fn save_signing_secret(&self, keypair: T) -> Result<()>;
    fn list_secrets(&self) -> Result<Vec<T>>;
}
