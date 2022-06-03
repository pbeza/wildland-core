mod swift_admin_manager;

use super::{EmptyResult, SeedPhraseResult};
use crate::{
    api::{AdminManagerError, SeedPhrase},
    ffi::identity::*,
};
use swift_admin_manager::AdminManager;
use swift_admin_manager::*;

#[swift_bridge::bridge]
mod ffi_bridge {
    extern "Rust" {
        fn create_seed_phrase() -> SeedPhraseResult;
        fn create_admin_manager() -> AdminManager;

        type AdminManager;
        fn get_master_identity(self: &mut AdminManager) -> OptionalIdentity;
        fn create_master_identity_from_seed_phrase(
            self: &mut AdminManager,
            name: String,
            seed: &SeedPhrase,
        ) -> IdentityResult;
        fn set_email(self: &mut AdminManager, email: String);
        fn send_verification_code(self: &mut AdminManager) -> EmptyResult;
        fn verify_email(self: &mut AdminManager, input_verification_code: String) -> EmptyResult;

        type EmptyResult;
        fn is_ok(self: &EmptyResult) -> bool;
        fn unwrap_err(self: &EmptyResult) -> AdminManagerError;

        type SeedPhraseResult;
        fn is_ok(self: &SeedPhraseResult) -> bool;
        fn unwrap(self: &SeedPhraseResult) -> SeedPhrase;
        fn unwrap_err(self: &SeedPhraseResult) -> AdminManagerError;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        type DynIdentity;
        fn set_name(self: &DynIdentity, name: String);
        fn get_name(self: &DynIdentity) -> String;

        type IdentityResult;
        fn unwrap(self: &IdentityResult) -> DynIdentity;

        type OptionalIdentity;
        fn is_some(self: &OptionalIdentity) -> bool;
        fn unwrap(self: &OptionalIdentity) -> DynIdentity;

        type AdminManagerError;
        fn to_string(self: &AdminManagerError) -> String;
        fn code(self: &AdminManagerError) -> u32;
    }
}
