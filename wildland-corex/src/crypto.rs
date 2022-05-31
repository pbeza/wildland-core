use crate::CoreXError;
use wildland_crypto::identity::{self, Identity, SeedPhraseWords};

pub fn try_identity_from_seed(seed: &SeedPhraseWords) -> Result<Identity, CoreXError> {
    Identity::try_from(seed).map_err(CoreXError::from)
}

pub fn generate_random_seed_phrase() -> Result<SeedPhraseWords, CoreXError> {
    identity::generate_random_seed_phrase().map_err(CoreXError::from)
}
