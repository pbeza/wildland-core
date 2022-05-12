use crate::api;
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

impl From<SeedPhrase> for crypto_identity::Identity {
    fn from(words: SeedPhrase) -> Self {
        crypto_identity::Identity::from_mnemonic(
            Mnemonic::from_str(
                &words
                    .0
                    .into_iter()
                    .intersperse(" ".into())
                    .collect::<String>(),
            )
            .unwrap(), // TODO handle err
        )
    }
}
