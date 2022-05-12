mod identity;
mod seed_phrase;

use crate::api::{self, SeedPhraseWords};
pub use identity::Identity;
pub use seed_phrase::SeedPhrase;
use wildland_crypto::identity as crypto_identity;

#[derive(Default)]
pub struct AdminManager<I: api::Identity> {
    identity: I,
}

impl AdminManager<Identity> {
    fn create_identity(
        &mut self,
        identity_type: api::IdentityType,
        name: String,
        inner_identity: crypto_identity::Identity,
    ) -> Identity {
        let identity = Identity::new(identity_type, name, inner_identity);
        self.identity = identity.clone();
        identity
    }
}

impl api::AdminManager<Identity> for AdminManager<Identity> {
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: SeedPhraseWords,
    ) -> Identity {
        self.create_identity(
            api::IdentityType::Master,
            name,
            SeedPhrase::from(seed).into(),
        )
    }

    fn create_master_identity(&mut self, name: String) -> Identity {
        self.create_identity(
            api::IdentityType::Master,
            name,
            *crypto_identity::from_random_seed().unwrap(),
        )
    }

    fn create_device_identity_from_seed_phrase(&mut self, name: String) -> Identity {
        self.create_identity(
            api::IdentityType::Device,
            name,
            *crypto_identity::from_random_seed().unwrap(),
        )
    }

    fn create_device_identity(&mut self, name: String, seed: SeedPhraseWords) -> Identity {
        self.create_identity(
            api::IdentityType::Master,
            name,
            SeedPhrase::from(seed).into(),
        )
    }

    fn get_identity(&self) -> Identity {
        self.identity.clone()
    }
}
