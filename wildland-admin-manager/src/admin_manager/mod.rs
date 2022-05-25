mod identity;

use crate::api::{self, AdminManagerError, IdentityApi, SeedPhrase};
pub use identity::Identity;

#[derive(Default)]
pub struct AdminManager {
    // TODO do we want to store more than one master identity
    // TODO do we want to keep mappings between a master identity and a set of device identities
    master_identity: Option<Box<dyn IdentityApi>>,
}

impl api::AdminManager for AdminManager {
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: &SeedPhrase,
    ) -> api::AdminManagerResult<Box<dyn IdentityApi>> {
        let identity = Identity::new(
            api::IdentityType::Master,
            name,
            wildland_corex::try_identity_from_seed(seed.as_ref())?,
        );
        self.master_identity = Some(Box::new(identity.clone())); // TODO Can user have multiple master identities? If not should it be overwritten?
        Ok(Box::new(identity)) // TODO return ref
    }

    // fn create_device_identity_from_seed_phrase(
    //     &mut self,
    //     name: String,
    //     seed: &SeedPhrase,
    // ) -> api::AdminManagerResult<Identity> {
    //     // let identity = Identity::new(
    //     //     api::IdentityType::Device,
    //     //     name,
    //     //     wildland_corex::try_identity_from_seed(seed.as_ref())?,
    //     // );
    //     let identity = Identity::new(api::IdentityType::Device, name, seed);
    //     // TODO keep it somehow?
    //     Ok(identity) // TODO return ref
    // }

    fn create_seed_phrase() -> api::AdminManagerResult<SeedPhrase> {
        wildland_corex::generate_random_seed_phrase()
            .map_err(AdminManagerError::from)
            .map(SeedPhrase::from)
    }

    fn get_master_identity(&mut self) -> &mut Option<Box<dyn IdentityApi>> {
        &mut self.master_identity
    }
}
