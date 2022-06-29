use crate::CoreXError;
use wildland_crypto::identity::{self, Identity, SeedPhraseWordsArray};

pub fn try_identity_from_seed(seed: &SeedPhraseWordsArray) -> Result<Identity, CoreXError> {
    Identity::try_from(seed).map_err(CoreXError::from)
}

pub fn generate_random_seed_phrase() -> Result<SeedPhraseWordsArray, CoreXError> {
    identity::generate_random_seed_phrase().map_err(CoreXError::from)
}

#[derive(Debug, Clone)]
pub struct SeedPhrase(SeedPhraseWordsArray);

impl SeedPhrase {
    pub fn get_string(&self) -> String {
        self.0.join(" ")
    }

    pub fn get_vec(&self) -> Vec<String> {
        self.0.clone().into()
    }
}

impl From<SeedPhraseWordsArray> for SeedPhrase {
    fn from(seed: SeedPhraseWordsArray) -> Self {
        Self(seed)
    }
}

impl From<SeedPhrase> for SeedPhraseWordsArray {
    fn from(seed: SeedPhrase) -> Self {
        seed.0
    }
}

impl From<&SeedPhrase> for SeedPhraseWordsArray {
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

impl AsRef<SeedPhraseWordsArray> for SeedPhrase {
    fn as_ref(&self) -> &SeedPhraseWordsArray {
        &self.0
    }
}
