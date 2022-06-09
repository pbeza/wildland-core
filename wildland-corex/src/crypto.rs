use crate::CoreXError;
use wildland_crypto::identity::{self, Identity, SeedPhraseWords};
use wildland_wallet::{FileWallet, ManifestSigningKeypair, Wallet};

pub enum WalletType {
    File,
}

pub fn list_keypairs(wallet: WalletType) -> Result<Vec<ManifestSigningKeypair>, CoreXError> {
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

#[derive(Debug, Clone)]
pub struct SeedPhrase([String; 12]);

impl SeedPhrase {
    pub fn get_string(&self) -> String {
        self.0.join(" ")
    }

    pub fn get_vec(&self) -> Vec<String> {
        self.0.clone().into()
    }
}

impl From<SeedPhraseWords> for SeedPhrase {
    fn from(seed: SeedPhraseWords) -> Self {
        Self(seed)
    }
}

impl From<SeedPhrase> for SeedPhraseWords {
    fn from(seed: SeedPhrase) -> Self {
        seed.0
    }
}

impl From<&SeedPhrase> for SeedPhraseWords {
    fn from(seed: &SeedPhrase) -> Self {
        seed.0.clone()
    }
}

impl TryFrom<Vec<String>> for SeedPhrase {
    type Error = CoreXError;

    fn try_from(vec: Vec<String>) -> Result<Self, Self::Error> {
        Ok(SeedPhrase(vec.try_into().map_err(|vec: Vec<_>| {
            CoreXError::ParseSeedPhraseError(vec.join(" "))
        })?))
    }
}

impl AsRef<[String; 12]> for SeedPhrase {
    fn as_ref(&self) -> &[String; 12] {
        &self.0
    }
}
