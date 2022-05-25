use super::{CxxDynIdentity, IdentityResult, OptionalIdentity, SeedPhraseResult};
use crate::{admin_manager::AdminManager, api::AdminManager as AdminManagerApi, api::SeedPhrase};

pub struct CxxAdminManager(AdminManager);

pub fn create_admin_manager() -> Box<CxxAdminManager> {
    Box::new(CxxAdminManager(AdminManager::default()))
}

pub fn create_seed_phrase() -> Box<SeedPhraseResult> {
    Box::new(AdminManager::create_seed_phrase().into())
}

impl CxxAdminManager {
    pub fn create_master_identity_from_seed_phrase(
        self: &mut CxxAdminManager,
        name: String,
        seed: &SeedPhrase,
    ) -> Box<IdentityResult> {
        let inner = self.0.create_master_identity_from_seed_phrase(name, seed);
        Box::new(inner.into())
    }

    pub fn get_master_identity(self: &mut CxxAdminManager) -> Box<OptionalIdentity> {
        let id = self.0.get_master_identity().as_mut().map(CxxDynIdentity);
        Box::new(id.into())
    }
}
