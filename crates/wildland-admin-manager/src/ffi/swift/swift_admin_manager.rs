use crate::{
    admin_manager::AdminManager as RustAdminManager,
    api::{AdminManager as AdminManagerApi, SeedPhrase},
    ffi::{identity::*, EmptyResult, SeedPhraseResult},
};

pub struct AdminManager(RustAdminManager);

pub fn create_admin_manager() -> AdminManager {
    AdminManager(RustAdminManager::default())
}

pub fn create_seed_phrase() -> SeedPhraseResult {
    RustAdminManager::create_seed_phrase().into()
}

impl AdminManager {
    pub fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: &SeedPhrase,
    ) -> IdentityResult {
        self.0
            .create_master_identity_from_seed_phrase(name, seed)
            .map(DynIdentity)
            .into()
    }

    pub fn get_master_identity(&self) -> OptionalIdentity {
        self.0.get_master_identity().map(DynIdentity).into()
    }

    pub fn set_email(&mut self, email: String) {
        self.0.set_email(email)
    }

    pub fn send_verification_code(self: &mut AdminManager) -> EmptyResult {
        self.0.send_verification_code().into()
    }

    pub fn verify_email(self: &mut AdminManager, input_verification_code: String) -> EmptyResult {
        self.0.verify_email(input_verification_code).into()
    }
}
