use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use super::wildland::{WildlandIdentity, WildlandIdentityApi, WildlandIdentityType};
use wildland_crypto::identity::{Identity, SeedPhraseWords};

use crate::crypto::{SeedPhrase, WalletType};
use crate::{CoreXError, CryptoSigningKeypair};

pub trait MasterIdentityApi: Display {
    fn get_seed_phrase(&self) -> SeedPhraseWords;
    fn get_signing_keypair(&self) -> Box<dyn CryptoSigningKeypair>;
    fn create_wildland_identity(
        &self,
        identity_type: WildlandIdentityType,
    ) -> Result<Arc<Mutex<dyn WildlandIdentityApi>>, CoreXError>;
}

#[derive(Clone)]
pub struct MasterIdentity {
    inner_identity: Identity,
}

impl MasterIdentity {
    pub fn default() -> Result<Self, CoreXError> {
        let seed = crate::generate_random_seed_phrase()
            .map_err(CoreXError::from)
            .map(SeedPhrase::from)?;

        let inner_identity =
            crate::try_identity_from_seed(seed.as_ref()).map_err(CoreXError::from)?;

        Ok(MasterIdentity::new(inner_identity))
    }

    pub fn new(inner_identity: Identity) -> Self {
        Self { inner_identity }
    }
}

impl MasterIdentityApi for MasterIdentity {
    fn get_seed_phrase(&self) -> SeedPhraseWords {
        self.inner_identity.get_seed_phrase()
    }

    fn get_signing_keypair(&self) -> Box<dyn CryptoSigningKeypair> {
        Box::new(self.inner_identity.signing_key())
    }

    fn create_wildland_identity(
        &self,
        identity_type: WildlandIdentityType,
    ) -> Result<Arc<Mutex<dyn WildlandIdentityApi>>, CoreXError> {
        let keypair = self.get_signing_keypair().into();
        let identity = WildlandIdentity::new(identity_type, keypair);

        identity.save(WalletType::File)?;

        Ok(Arc::new(Mutex::new(identity)))
    }
}

impl Display for MasterIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Type: {:?}
Seed phrase: {}
",
            "Master",
            self.inner_identity.get_seed_phrase().join(" ")
        )
    }
}
