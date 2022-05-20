use wildland_corex::SeedPhraseWords as CorexSeedPhrase;

#[derive(Debug, Clone)]
pub struct SeedPhrase([String; 12]);

impl SeedPhrase {
    pub fn get_string(&self) -> String {
        self.0.join(" ")
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
    type Error = Vec<String>;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        todo!()
    }
}
