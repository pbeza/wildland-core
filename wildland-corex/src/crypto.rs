use crate::CoreXError;
use wildland_crypto::identity::{self, keys::SigningKeyPair, Identity, SeedPhraseWords};
use wildland_wallet::{FileWallet, ManifestSigningKeypair, SigningKeyType, Wallet};

pub enum WalletType {
    File,
}

pub fn save_identity(identity: &Identity, wallet: WalletType) -> Result<(), CoreXError> {
    match &wallet {
        WalletType::File => {
            let keypair = ManifestSigningKeypair::from_keys(
                SigningKeyType::Master,
                identity.signing_key().seckey_as_bytes(),
                identity.signing_key().pubkey_as_bytes(),
            );

            let wallet = FileWallet::new().map_err(|e| {
                CoreXError::IdentityGenerationError(format!("Could not instantiate Wallet. {}", e))
            })?;

            wallet
                .save_signing_secret(keypair)
                .map_err(|e| CoreXError::IdentityGenerationError(e.to_string()))?
        }
    }

    Ok(())
}

pub fn list_keypairs(wallet: WalletType) -> Result<Vec<ManifestSigningKeypair>, CoreXError> {
    // pub fn list_keypairs(wallet: WalletType) -> Result<Vec<KeyPair>, CoreXError> {
    match &wallet {
        WalletType::File => {
            let wallet = FileWallet::new().map_err(|e| {
                CoreXError::IdentityGenerationError(format!("Could not instantiate Wallet. {}", e))
            })?;

            let manifest_keypairs = wallet
                .list_secrets()
                .map_err(|e| CoreXError::IdentityReadError(e.to_string()))?;

            Ok(manifest_keypairs)
        }
    }
}

pub fn try_identity_from_seed(seed: &SeedPhraseWords) -> Result<Identity, CoreXError> {
    Identity::try_from(seed).map_err(CoreXError::from)
}

pub fn generate_random_seed_phrase() -> Result<SeedPhraseWords, CoreXError> {
    identity::generate_random_seed_phrase().map_err(CoreXError::from)
}
