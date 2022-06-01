use crate::admin_manager::AdminManager;
use crate::api::AdminManager as AdminManagerTrait;
use crate::api::Identity;
use crate::api::{AdminManagerError, AdminManagerResult, SeedPhrase};
use ffi_macro::binding_wrapper;

// Those two types are required in binding_wrapper.
type ResultFfi<T> = AdminManagerResult<T>;
type ResultFfiError = AdminManagerError;


#[binding_wrapper]
mod ffi_binding {
    extern "Rust" {
        // fn create_master_identity_from_seed_phrase(
        //     self: &mut AdminManager,
        //     name: String,
        //     seed: &SeedPhrase,
        // ) -> Result<dyn Identity>;
        fn create_admin_manager() -> AdminManager;
        // fn get_master_identity(self: &AdminManager) -> Option<Arc<dyn Identity>>;
        // fn send_verification_code(self: &mut AdminManager) -> AdminManagerResult<()>;
        fn set_email(self: &mut AdminManager, email: String);
        // fn verify_email(self: &mut AdminManager, verification_code: String) -> AdminManagerResult<()>;

        // fn create_seed_phrase() -> Result<SeedPhrase>;
        // fn get_string(self: &SeedPhrase) -> String;
        // fn get_vec(self: &SeedPhrase) -> Vec<String>;

        // fn get_identity_type(self: &dyn Identity) -> IdentityType;
        // fn get_name(self: &dyn Identity) -> String;
        // fn set_name(self: &mut dyn Identity, name: String);
        // fn get_pubkey(self: &dyn Identity) -> Vec<u8>;
        // fn get_fingerprint(self: &dyn Identity) -> Vec<u8>;
        // fn get_seed_phrase(self: &dyn Identity) -> SeedPhraseWords;    // Translate slice into vector for FFI purpose.

        // fn to_string(self: &AdminManagerError) -> String;
        // fn code(self: &AdminManagerError) -> u32;

    }
}
