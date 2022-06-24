use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use wildland_crypto::identity::SigningKeypair;
use xdg::BaseDirectories;

use crate::{ManifestSigningKeypair, SigningKeyType, Wallet, WalletError};

#[derive(Clone, Debug)]
pub struct FileWallet {
    base_directory: BaseDirectories,
}

impl FileWallet {
    fn write_secret_file(&self, name: String, contents: String) -> Result<(), WalletError> {
        let file = self
            .base_directory
            .place_data_file(name)
            .map_err(|e| WalletError::FileError(e.to_string()))?;

        fs::write(&file, contents).map_err(|e| WalletError::FileError(e.to_string()))
    }
}

pub fn create_file_wallet() -> Result<Box<dyn Wallet>, WalletError> {
    Ok(Box::new(FileWallet {
        base_directory: BaseDirectories::with_prefix("wildland/wallet")
            .map_err(|e| WalletError::FileError(e.to_string()))?,
    }))
}

#[derive(Serialize, Deserialize)]
struct WalletKeyFileContents {
    privkey: String,
    pubkey: String,
    key_type: SigningKeyType,
}

impl Wallet for FileWallet {
    fn save_signing_secret(&self, keypair: ManifestSigningKeypair) -> Result<(), WalletError> {
        self.write_secret_file(
            format!("{}.json", keypair.fingerprint()),
            json!(WalletKeyFileContents {
                privkey: hex::encode(keypair.get_private_key()),
                pubkey: hex::encode(keypair.get_public_key()),
                key_type: keypair.get_key_type(),
            })
            .to_string(),
        )
    }

    fn list_secrets(&self) -> Result<Vec<ManifestSigningKeypair>, WalletError> {
        self.base_directory
            .list_data_files(".")
            .into_iter()
            .map(|f| {
                let contents =
                    fs::read_to_string(f).map_err(|e| WalletError::FileError(e.to_string()))?;
                let file_data: WalletKeyFileContents = serde_json::from_str(&contents)
                    .map_err(|e| WalletError::FileError(e.to_string()))?;

                Ok(ManifestSigningKeypair::from_keypair(
                    file_data.key_type,
                    SigningKeypair::try_from_str(&file_data.privkey, &file_data.pubkey).unwrap(), // TODO get rid off unwrap
                ))
            })
            .collect()
    }
}
