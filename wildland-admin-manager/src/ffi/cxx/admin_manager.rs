use super::{EmptyResult, SeedPhraseResult};
use crate::{
    admin_manager::{self, AdminManager as RustAdminManager},
    api::{AdminManager as AdminManagerApi, SeedPhrase},
    ffi::{
        email_client::BoxedDynEmailClient,
        identity::{DynIdentity, IdentityResult, OptionalIdentity},
    },
};

// TODO in the future we must provide some mock for testing on the clients side and an actual implementation
pub struct AdminManager(RustAdminManager);

pub fn create_admin_manager(email_client: &BoxedDynEmailClient) -> Box<AdminManager> {
    Box::new(AdminManager(RustAdminManager::new(
        (**email_client).0.clone(),
    )))
}

pub fn create_seed_phrase() -> Box<SeedPhraseResult> {
    Box::new(admin_manager::create_seed_phrase().into())
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

    pub fn send_verification_code(self: &mut AdminManager) -> Box<EmptyResult> {
        Box::new(self.0.send_verification_code().into())
    }

    pub fn verify_email(
        self: &mut AdminManager,
        input_verification_code: String,
    ) -> Box<EmptyResult> {
        Box::new(self.0.verify_email(input_verification_code).into())
    }
}
