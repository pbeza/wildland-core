mod cxx_result;

use self::cxx_result::CxxResult;
use crate::{
    admin_manager::{AdminManager, Identity},
    api::{AdminManager as AdminManagerApi, AdminManagerError, SeedPhrase},
};

type AdminManagerType = AdminManager<Identity>;
type SeedPhraseResult = CxxResult<SeedPhrase>;
// type OptionalIdentity = CxxOption<Identity>;

fn create_admin_manager() -> Box<AdminManagerType> {
    Box::new(AdminManager::<Identity>::default())
}

fn create_seed_phrase() -> Box<SeedPhraseResult> {
    Box::new(AdminManager::<Identity>::create_seed_phrase().into())
}

#[cxx::bridge(namespace = "cargo::api")]
mod api {
    extern "Rust" {
        type AdminManagerType;
        // type OptionalIdentity;
        fn create_admin_manager() -> Box<AdminManagerType>;
        // fn get_master_identity(self: &AdminManagerType) -> OptionalIdentity;

        type SeedPhraseResult;
        fn create_seed_phrase() -> Box<SeedPhraseResult>;
        fn is_ok(self: &SeedPhraseResult) -> bool;
        fn unwrap(self: &SeedPhraseResult) -> &SeedPhrase;
        fn unwrap_err(self: &SeedPhraseResult) -> &AdminManagerError;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;

        type AdminManagerError;
        fn to_string(self: &AdminManagerError) -> String;
        fn code(self: &AdminManagerError) -> u32;

    }
}
