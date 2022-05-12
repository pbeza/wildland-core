mod identity;
mod seed_phrase;

use crate::api::{self, SeedPhraseWords};
use anyhow::Result;
pub use identity::Identity;
pub use seed_phrase::SeedPhrase;
use wildland_crypto::identity as crypto_identity;

pub struct AdminManager<I: api::Identity> {
    master_identity: Option<I>,
}

impl Default for AdminManager<Identity> {
    fn default() -> Self {
        Self {
            master_identity: Default::default(),
        }
    }
}

impl api::AdminManager<Identity> for AdminManager<Identity> {
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: SeedPhraseWords,
    ) -> Result<Identity> {
        let identity = Identity::new(
            api::IdentityType::Master,
            name,
            SeedPhrase::try_from(seed)?.try_into()?,
        );
        self.master_identity = Some(identity.clone()); // TODO Can user have multiple master identities? If not should it be overwritten?
        Ok(identity)
    }

    fn create_device_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: SeedPhraseWords,
    ) -> Result<Identity> {
        let identity = Identity::new(
            api::IdentityType::Device,
            name,
            SeedPhrase::try_from(seed)?.try_into()?,
        );
        // TODO add
        Ok(identity)
    }

    fn create_seed_phrase() -> Result<SeedPhraseWords> {
        crypto_identity::generate_random_seed_phrase()
    }

    fn get_master_identity(&self) -> Option<Identity> {
        self.master_identity.clone()
    }
}
