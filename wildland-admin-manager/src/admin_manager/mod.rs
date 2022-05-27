mod identity;

use crate::api::{self, AdminManagerError, Identity, SeedPhrase};
pub use identity::CryptoIdentity;
use std::sync::Arc;

#[derive(Default)]
pub struct AdminManager {
    // TODO do we want to store more than one master identity
    // TODO do we want to keep mappings between a master identity and a set of device identities
    master_identity: Option<Arc<dyn Identity>>,
}

impl api::AdminManager for AdminManager {
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: &SeedPhrase,
    ) -> api::AdminManagerResult<Arc<dyn Identity>> {
        let identity = CryptoIdentity::new(
            api::IdentityType::Master,
            name,
            wildland_corex::try_identity_from_seed(seed.as_ref())?,
        );
        self.master_identity = Some(Arc::new(identity)); // TODO Can user have multiple master identities? If not should it be overwritten?
        Ok(self.master_identity.as_ref().unwrap().clone())
    }

    // }

    fn create_seed_phrase() -> api::AdminManagerResult<SeedPhrase> {
        wildland_corex::generate_random_seed_phrase()
            .map_err(AdminManagerError::from)
            .map(SeedPhrase::from)
    }

    fn get_master_identity(&self) -> Option<Arc<dyn Identity>> {
        self.master_identity.clone()
    }
}
