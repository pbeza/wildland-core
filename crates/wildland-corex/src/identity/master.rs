use super::wildland::{WildlandIdentity, WildlandIdentityType};
use crate::{crypto::SeedPhrase, CoreXError};
use std::fmt::Display;
use wildland_crypto::identity::{Identity, SeedPhraseWordsArray, SigningKeypair};

pub struct MasterIdentity {
    inner_identity: Identity,
}

impl MasterIdentity {
    pub fn new() -> Result<Self, CoreXError> {
        let seed = crate::generate_random_seed_phrase()
            .map_err(CoreXError::from)
            .map(SeedPhrase::from)?;

        let inner_identity =
            crate::try_identity_from_seed(seed.as_ref()).map_err(CoreXError::from)?;

        Ok(Self::with_identity(inner_identity))
    }

    pub fn with_identity(inner_identity: Identity) -> Self {
        Self { inner_identity }
    }

    pub fn get_seed_phrase(&self) -> SeedPhraseWordsArray {
        self.inner_identity.get_seed_phrase()
    }

    pub fn get_forest_keypair(&self) -> SigningKeypair {
        self.inner_identity.forest_keypair(0)
    }

    pub fn create_wildland_identity(
        &self,
        identity_type: WildlandIdentityType,
        name: String,
    ) -> Result<WildlandIdentity, CoreXError> {
        let keypair = self.get_forest_keypair();
        let identity = WildlandIdentity::new(identity_type, keypair, name);

        Ok(identity)
    }
}

impl Display for MasterIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Type: Master
Seed phrase: {}
",
            self.inner_identity.get_seed_phrase().join(" ")
        )
    }
}
