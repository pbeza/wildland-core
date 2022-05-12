use crate::api;
use anyhow::anyhow;
use bip39::Mnemonic;
use std::str::FromStr;
use wildland_crypto::identity as crypto_identity;

#[derive(Default, Clone)]
pub struct SeedPhrase(api::SeedPhraseWords);

impl From<api::SeedPhraseWords> for SeedPhrase {
    fn from(words: api::SeedPhraseWords) -> Self {
        Self(words)
    }
}

impl TryFrom<SeedPhrase> for crypto_identity::Identity {
    type Error = anyhow::Error;

    fn try_from(words: SeedPhrase) -> Result<Self, Self::Error> {
        Ok(crypto_identity::Identity::from_mnemonic(
            Mnemonic::from_str(&words.0.join(" ")).map_err(|e| anyhow!(e))?,
        ))
    }
}
