mod cxx_admin_manager;

use crate::{
    api::{AdminManagerError, SeedPhrase},
    ffi::{identity::*, SeedPhraseResult},
};
use cxx_admin_manager::*;

#[allow(clippy::needless_lifetimes)]
#[cxx::bridge(namespace = "wildland")]
mod ffi_cxx {
    extern "Rust" {
        fn create_seed_phrase() -> Box<SeedPhraseResult>;
        fn create_admin_manager() -> Box<AdminManager>;

        type AdminManager;
        fn get_master_identity(self: &mut AdminManager) -> Box<OptionalIdentity>;
        fn create_master_identity_from_seed_phrase(
            self: &mut AdminManager,
            name: String,
            seed: &SeedPhrase,
        ) -> Box<IdentityResult>;

        type SeedPhraseResult;
        fn is_ok(self: &SeedPhraseResult) -> bool;
        fn unwrap(self: &SeedPhraseResult) -> &SeedPhrase;
        fn unwrap_err(self: &SeedPhraseResult) -> &AdminManagerError;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        type DynIdentity;
        fn set_name(self: &DynIdentity, name: String);
        fn get_name(self: &DynIdentity) -> String;

        type IdentityResult;
        unsafe fn unwrap(self: &IdentityResult) -> &DynIdentity;

        type OptionalIdentity;
        fn is_some(self: &OptionalIdentity) -> bool;
        unsafe fn unwrap(self: &OptionalIdentity) -> &DynIdentity;

        type AdminManagerError;
        fn to_string(self: &AdminManagerError) -> String;
        fn code(self: &AdminManagerError) -> u32;
    }
}
