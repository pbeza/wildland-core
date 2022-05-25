mod cxx_admin_manager;
mod cxx_identity;
mod cxx_option;
mod cxx_result;

use crate::api::{AdminManagerError, SeedPhrase};
use cxx_admin_manager::*;
use cxx_identity::*;
use cxx_result::*;

type SeedPhraseResult = CxxResult<SeedPhrase>;

#[allow(clippy::needless_lifetimes)]
#[cxx::bridge(namespace = "cargo::api")]
mod api {
    extern "Rust" {
        type CxxAdminManager;
        fn create_admin_manager() -> Box<CxxAdminManager>;
        fn get_master_identity(self: &mut CxxAdminManager) -> Box<OptionalIdentity>;
        unsafe fn create_master_identity_from_seed_phrase<'a>(
            self: &'a mut CxxAdminManager,
            name: String,
            seed: &SeedPhrase,
        ) -> Box<IdentityResult<'a>>;

        type SeedPhraseResult;
        fn create_seed_phrase() -> Box<SeedPhraseResult>;
        fn is_ok(self: &SeedPhraseResult) -> bool;
        fn unwrap(self: &SeedPhraseResult) -> &SeedPhrase;
        fn unwrap_err(self: &SeedPhraseResult) -> &AdminManagerError;

        type IdentityResult<'a>;
        unsafe fn unwrap_mut<'a>(self: &'a mut IdentityResult<'a>) -> &'a mut CxxDynIdentity<'a>;
        type OptionalIdentity<'a>;
        type DynIdentity;
        type CxxDynIdentity<'a>;
        fn is_some(self: &OptionalIdentity) -> bool;
        unsafe fn unwrap_mut<'a>(self: &'a mut OptionalIdentity<'a>) -> &'a mut CxxDynIdentity;
        fn set_name(self: &mut CxxDynIdentity, name: String);
        fn get_name(self: &CxxDynIdentity) -> String;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        type AdminManagerError;
        fn to_string(self: &AdminManagerError) -> String;
        fn code(self: &AdminManagerError) -> u32;

    }
}
