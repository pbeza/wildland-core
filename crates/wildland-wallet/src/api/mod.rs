use crate::{keys::sign::ManifestSigningKeypair, WalletError};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SigningKeyType {
    Forest,
    Device,
}

pub trait Wallet: Debug {
    fn save_signing_secret(&self, keypair: ManifestSigningKeypair) -> Result<(), WalletError>;
    fn list_secrets(&self) -> Result<Vec<ManifestSigningKeypair>, WalletError>;
}
