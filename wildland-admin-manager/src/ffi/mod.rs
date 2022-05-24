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
type IdentityResult = CxxResult<Identity>;
pub type OptionalIdentity<'a> = CxxRefOption<'a, Identity>;

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
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        type AdminManagerError;
        fn to_string(self: &AdminManagerError) -> String;
        fn code(self: &AdminManagerError) -> u32;

    }
}
