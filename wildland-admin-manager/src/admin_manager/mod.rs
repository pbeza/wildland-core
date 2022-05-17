mod identity;

use api::{AdminManagerError, CoreXError};
pub use identity::Identity;
use wildland_admin_manager_api as api;
use wildland_crypto::identity as crypto_identity;

pub struct AdminManager<I: api::Identity> {
    // TODO do we want to store more than one master identity
    // TODO do we want to keep mappings between a master identity and a set of device identities
    master_identity: Option<I>,
}

impl Default for AdminManager<Identity> {
    fn default() -> Self {
        Self {
            master_identity: Default::default(),
        }
    }
}

impl api::AdminManager for AdminManager<Identity> {
    type Identity = Identity;

    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: api::SeedPhraseWords,
    ) -> api::AdminManagerResult<Identity> {
        let identity = Identity::new(
            api::IdentityType::Master,
            name,
            seed.try_into()
                .map_err(|e| AdminManagerError::CoreX(CoreXError::Crypto(e)))?, // TODO delegate to corex ?
        );
        self.master_identity = Some(identity.clone()); // TODO Can user have multiple master identities? If not should it be overwritten?
        Ok(identity)
    }

    fn create_device_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: api::SeedPhraseWords,
    ) -> api::AdminManagerResult<Identity> {
        let identity = Identity::new(
            api::IdentityType::Device,
            name,
            seed.try_into()
                .map_err(|e| AdminManagerError::CoreX(CoreXError::Crypto(e)))?, // TODO delegate to corex ?
        );
        // TODO keep it somehow?
        Ok(identity)
    }

    fn create_seed_phrase() -> api::AdminManagerResult<api::SeedPhraseWords> {
        crypto_identity::generate_random_seed_phrase()
            .map_err(|e| AdminManagerError::CoreX(CoreXError::Crypto(e))) // TODO delegate to corex ?
    }

    fn get_master_identity(&self) -> Option<Identity> {
        self.master_identity.clone()
    }
}
