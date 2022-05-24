pub mod cxx_option;
mod cxx_result;

use self::{cxx_option::CxxRefOption, cxx_result::CxxResult};
use crate::{
    admin_manager::{AdminManager, Identity},
    api::{AdminManager as AdminManagerApi, AdminManagerError, SeedPhrase},
};

struct CxxAdminManager(AdminManager);
type SeedPhraseResult = CxxResult<SeedPhrase>;
type IdentityResult = CxxResult<Identity>;
pub type OptionalIdentity<'a> = CxxRefOption<'a, Identity>;

fn create_admin_manager() -> Box<CxxAdminManager> {
    Box::new(CxxAdminManager(AdminManager::default()))
}

fn create_seed_phrase() -> Box<SeedPhraseResult> {
    Box::new(AdminManager::create_seed_phrase().into())
}

impl CxxAdminManager {
    fn create_master_identity_from_seed_phrase(
        self: &mut CxxAdminManager,
        name: String,
        seed: &SeedPhrase,
    ) -> Box<IdentityResult> {
        Box::new(
            self.0
                .create_master_identity_from_seed_phrase(name, seed)
                .into(),
        )
    }

    fn get_master_identity(self: &mut CxxAdminManager) -> Box<OptionalIdentity> {
        Box::new(self.0.get_master_identity().into())
    }
}

// #[allow(clippy::needless_lifetimes)] // TODO check it
#[cxx::bridge(namespace = "cargo::api")]
mod api {
    extern "Rust" {
        type CxxAdminManager;
        fn create_admin_manager() -> Box<CxxAdminManager>;
        fn get_master_identity(self: &mut CxxAdminManager) -> Box<OptionalIdentity>;
        fn create_master_identity_from_seed_phrase(
            self: &mut CxxAdminManager,
            name: String,
            seed: &SeedPhrase,
        ) -> Box<IdentityResult>;

        type SeedPhraseResult;
        fn create_seed_phrase() -> Box<SeedPhraseResult>;
        fn is_ok(self: &SeedPhraseResult) -> bool;
        fn unwrap(self: &SeedPhraseResult) -> &SeedPhrase;
        fn unwrap_err(self: &SeedPhraseResult) -> &AdminManagerError;

        type Identity;
        type IdentityResult;
        type OptionalIdentity<'a>;
        fn is_some(self: &OptionalIdentity) -> bool;
        unsafe fn unwrap<'a>(self: &'a mut OptionalIdentity<'a>) -> &'a mut Identity;
        fn set_name(self: &mut Identity, name: String);
        fn get_name(self: &Identity) -> String;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;

        type AdminManagerError;
        fn to_string(self: &AdminManagerError) -> String;
        fn code(self: &AdminManagerError) -> u32;

    }
}
