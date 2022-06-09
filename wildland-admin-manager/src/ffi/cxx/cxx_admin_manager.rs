use super::{EmptyResult, SeedPhraseResult};
use crate::{
    admin_manager::AdminManager as RustAdminManager,
    api::AdminManager as AdminManagerApi,
    api::SeedPhrase,
    ffi::identity::{DynIdentity, IdentityResult, OptionalIdentity},
};

pub struct AdminManager(RustAdminManager);

pub fn create_admin_manager() -> Box<AdminManager> {
    Box::new(AdminManager(RustAdminManager::default()))
}

pub fn create_seed_phrase() -> Box<SeedPhraseResult> {
    Box::new(RustAdminManager::create_seed_phrase().into())
}

impl AdminManager {
    pub fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: &SeedPhrase,
    ) -> Box<IdentityResult> {
        let inner = self
            .0
            .create_master_identity_from_seed_phrase(name, seed)
            .map(DynIdentity);
        Box::new(inner.into())
    }

    pub fn get_master_identity(&self) -> Box<OptionalIdentity> {
        Box::new(self.0.get_master_identity().map(DynIdentity).into())
    }

    pub fn set_email(&mut self, email: String) {
        self.0.set_email(email)
    }

    pub fn request_verification_email(self: &mut AdminManager) -> Box<EmptyResult> {
        Box::new(self.0.request_verification_email().into())
    }

    pub fn verify_email(self: &mut AdminManager, verification_code: String) -> Box<EmptyResult> {
        Box::new(self.0.verify_email(verification_code).into())
    }
}
