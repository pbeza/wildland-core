use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use hex::FromHex;
use xdg::BaseDirectories;

use crate::{SigningKeyType, ManifestSigningKeypair, Wallet, SigningKeypair};

pub struct FileWallet {
    base_directory: BaseDirectories,
}

impl FileWallet {
    pub fn new() -> Result<Self> {
        Ok(FileWallet {
            base_directory: BaseDirectories::with_prefix("wildland/wallet")?,
        })
    }

    fn write_secret_file(&self, name: String, contents: String) -> Result<()> {
        let file = self.base_directory.place_data_file(name)?;

        Ok(fs::write(&file, contents)?)
    }
}

#[derive(Serialize, Deserialize)]
struct WalletKeyFileContents {
    privkey: String,
    pubkey: String,
    key_type: SigningKeyType,
}

impl Wallet<ManifestSigningKeypair> for FileWallet {
    fn save_signing_secret(&self, keypair: ManifestSigningKeypair) -> Result<()> {
        Ok(self
            .write_secret_file(
                format!("{}.json", keypair.fingerprint()),
                json!(WalletKeyFileContents {
                    privkey: hex::encode(keypair.get_private_key()),
                    pubkey: hex::encode(keypair.get_public_key()),
                    key_type: keypair.get_key_type(),
                })
                .to_string(),
            )
            .map_err(|err| {
                anyhow::Error::msg(format!("Could not write to secret file. {}.", err))
            })?)
    }

    fn list_secrets(&self) -> Result<Vec<ManifestSigningKeypair>> {
        let secrets: Vec<ManifestSigningKeypair> = self
            .base_directory
            .list_data_files(".")
            .into_iter()
            .map(|f| {
                let contents = fs::read_to_string(f).map_err(Error::msg)?;
                let file_data: WalletKeyFileContents =
                    serde_json::from_str(&contents).map_err(Error::msg)?;

                Ok(ManifestSigningKeypair::from_keys(
                    file_data.key_type,
                    <[u8; 32]>::from_hex(file_data.privkey).map_err(Error::msg)?,
                    <[u8; 32]>::from_hex(file_data.pubkey).map_err(Error::msg)?,
                ))
            })
            .collect::<Result<_, Error>>()?;

        Ok(secrets)
    }
}
