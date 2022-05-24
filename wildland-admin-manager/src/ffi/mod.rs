mod cxx_admin_manager;
mod cxx_option;
mod cxx_result;

use crate::{
    admin_manager::Identity,
    api::{AdminManagerError, SeedPhrase},
};
use cxx_admin_manager::*;
use cxx_option::*;
use cxx_result::*;

type SeedPhraseResult = CxxResult<SeedPhrase>;
type IdentityResult<'a> = CxxResult<&'a mut Identity>;
pub type OptionalIdentity<'a> = CxxOption<&'a mut Identity>;

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

        type Identity;
        type IdentityResult<'a>;
        unsafe fn unwrap_mut<'a>(self: &'a mut IdentityResult<'a>) -> &'a mut Identity;
        type OptionalIdentity<'a>;
        fn is_some(self: &OptionalIdentity) -> bool;
        unsafe fn unwrap_mut<'a>(self: &'a mut OptionalIdentity<'a>) -> &'a mut Identity;
        fn set_name(self: &mut Identity, name: String);
        fn get_name(self: &Identity) -> String;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        type AdminManagerError;
        fn to_string(self: &AdminManagerError) -> String;
        fn code(self: &AdminManagerError) -> u32;

    }
}
