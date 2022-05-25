mod identity;

use crate::api::{self, AdminManagerError, Identity, SeedPhrase};
pub use identity::CryptoIdentity;

#[derive(Default)]
pub struct AdminManager {
    // TODO do we want to store more than one master identity
    // TODO do we want to keep mappings between a master identity and a set of device identities
    master_identity: Option<Box<dyn Identity>>,
}

impl api::AdminManager for AdminManager {
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: &SeedPhrase,
    ) -> api::AdminManagerResult<&mut Box<dyn Identity>> {
        let identity = CryptoIdentity::new(
            api::IdentityType::Master,
            name,
            wildland_corex::try_identity_from_seed(seed.as_ref())?,
        );
        self.master_identity = Some(Box::new(identity)); // TODO Can user have multiple master identities? If not should it be overwritten?
        Ok(self.master_identity.as_mut().unwrap())
    }

    // }

    fn create_seed_phrase() -> api::AdminManagerResult<SeedPhrase> {
        wildland_corex::generate_random_seed_phrase()
            .map_err(AdminManagerError::from)
            .map(SeedPhrase::from)
    }

    fn get_master_identity(&mut self) -> &mut Option<Box<dyn Identity>> {
        &mut self.master_identity
    }
}
