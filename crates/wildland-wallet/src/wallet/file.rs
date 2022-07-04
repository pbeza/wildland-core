use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, path::PathBuf};
use wildland_crypto::identity::SigningKeypair;

use crate::{ManifestSigningKeypair, SigningKeyType, Wallet, WalletError};

#[derive(Clone, Debug)]
pub struct FileWallet {
    wallet_directory: PathBuf,
}

impl FileWallet {
    fn write_secret_file(&self, name: String, contents: String) -> Result<(), WalletError> {
        if !self.wallet_directory.exists() {
            fs::create_dir_all(&self.wallet_directory)
                .map_err(|e| WalletError::FileError(e.to_string()))?;
        }

        let file = self.wallet_directory.join(name);

        fs::write(&file, contents).map_err(|e| WalletError::FileError(e.to_string()))
    }
}

pub fn create_file_wallet() -> Result<Box<dyn Wallet>, WalletError> {
    let project_dirs = ProjectDirs::from("com", "wildland", "Cargo");

    if project_dirs.is_none() {
        return Err(WalletError::FileError(
            "Could not instantiate Wallet project directory".to_string(),
        ));
    }

    let wallet_dir = project_dirs.unwrap().data_local_dir().join("wallet");

    Ok(Box::new(FileWallet {
        wallet_directory: wallet_dir,
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
        if !self.wallet_directory.exists() {
            return Ok(vec![]);
        }

        let files = fs::read_dir(&self.wallet_directory)
            .map_err(|e| WalletError::FileError(e.to_string()))?;

        files
            .into_iter()
            .map(|f| {
                let dir_entry = f.map_err(|e| WalletError::FileError(e.to_string()))?;

                let contents = fs::read_to_string(dir_entry.path())
                    .map_err(|e| WalletError::FileError(e.to_string()))?;
                let file_data: WalletKeyFileContents = serde_json::from_str(&contents)
                    .map_err(|e| WalletError::FileError(e.to_string()))?;

                Ok(ManifestSigningKeypair::from_keypair(
                    file_data.key_type,
                    SigningKeypair::try_from_str(&file_data.privkey, &file_data.pubkey)
                        .map_err(WalletError::Crypto)?,
                ))
            })
            .collect()
    }
}
