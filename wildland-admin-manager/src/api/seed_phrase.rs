use wildland_corex::SeedPhraseWords as CorexSeedPhrase;

use super::AdminManagerError;

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

impl From<CorexSeedPhrase> for SeedPhrase {
    fn from(seed: CorexSeedPhrase) -> Self {
        Self(seed)
    }
}

impl From<SeedPhrase> for CorexSeedPhrase {
    fn from(seed: SeedPhrase) -> Self {
        seed.0
    }
}

impl TryFrom<Vec<String>> for SeedPhrase {
    type Error = AdminManagerError;

    fn try_from(vec: Vec<String>) -> Result<Self, Self::Error> {
        Ok(SeedPhrase(vec.try_into().map_err(|vec: Vec<_>| {
            AdminManagerError::ParseSeedPhraseError(vec.join(" "))
        })?))
    }
}

impl AsRef<[String; 12]> for SeedPhrase {
    fn as_ref(&self) -> &[String; 12] {
        &self.0
    }
}
