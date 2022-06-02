mod cxx_admin_manager;

use crate::{
    api::{AdminManagerError, SeedPhrase},
    ffi::{identity::*, EmptyResult, SeedPhraseResult},
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
            seed: &Box<SeedPhrase>,
        ) -> Box<IdentityResult>;
        fn set_email(self: &mut AdminManager, email: String);
        fn send_verification_code(self: &mut AdminManager) -> Box<EmptyResult>;
        fn verify_email(
            self: &mut AdminManager,
            input_verification_code: String,
        ) -> Box<EmptyResult>;

        type EmptyResult;
        fn is_ok(self: &EmptyResult) -> bool;
        fn boxed_unwrap_err(self: &EmptyResult) -> Box<AdminManagerError>;

        type SeedPhraseResult;
        fn is_ok(self: &SeedPhraseResult) -> bool;
        fn boxed_unwrap(self: &SeedPhraseResult) -> Box<SeedPhrase>;
        fn boxed_unwrap_err(self: &SeedPhraseResult) -> Box<AdminManagerError>;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        type DynIdentity;
        fn set_name(self: &DynIdentity, name: String);
        fn get_name(self: &DynIdentity) -> String;

        type IdentityResult;
        fn is_ok(self: &IdentityResult) -> bool;
        fn boxed_unwrap(self: &IdentityResult) -> Box<DynIdentity>;
        fn boxed_unwrap_err(self: &IdentityResult) -> Box<AdminManagerError>;

        type OptionalIdentity;
        fn is_some(self: &OptionalIdentity) -> bool;
        fn boxed_unwrap(self: &OptionalIdentity) -> Box<DynIdentity>;

        type AdminManagerError;
        fn to_string(self: &AdminManagerError) -> String;
        fn code(self: &AdminManagerError) -> u32;
    }
}
