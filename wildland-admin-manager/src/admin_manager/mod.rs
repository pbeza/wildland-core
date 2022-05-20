mod identity;

use api::AdminManagerError;
pub use identity::Identity;
use wildland_admin_manager_api as api;
use wildland_corex::SeedPhraseWords;

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
        seed: SeedPhraseWords,
    ) -> api::AdminManagerResult<Identity> {
        let identity = Identity::new(
            api::IdentityType::Master,
            name,
            wildland_corex::try_identity_from_seed(seed)?,
        );
        self.master_identity = Some(identity.clone()); // TODO Can user have multiple master identities? If not should it be overwritten?
        Ok(identity)
    }

    fn create_device_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: SeedPhraseWords,
    ) -> api::AdminManagerResult<Identity> {
        let identity = Identity::new(
            api::IdentityType::Device,
            name,
            wildland_corex::try_identity_from_seed(seed)?,
        );
        // TODO keep it somehow?
        Ok(identity)
    }

    fn create_seed_phrase() -> api::AdminManagerResult<SeedPhraseWords> {
        wildland_corex::generate_random_seed_phrase().map_err(AdminManagerError::from)
    }

    fn get_master_identity(&self) -> Option<Identity> {
        self.master_identity.clone()
    }
}
