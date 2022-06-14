use crate::admin_manager::AdminManager;
use crate::api::AdminManager as AdminManagerTrait;
use crate::api::Identity;
use crate::api::IdentityType;
use crate::api::{AdminManagerError, SeedPhrase};
use ffi_macro::binding_wrapper;
use wildland_corex::SeedPhraseWords;

// Define Error type and `()` type.
type ErrorType = AdminManagerError;
type VoidType = ();

#[binding_wrapper]
mod ffi_binding {
    use super::{AdminManagerTrait, Identity};
    extern "Rust" {
        type AdminManager;
        fn create_master_identity_from_seed_phrase(
            self: &mut AdminManager,
            name: String,
            seed: &SeedPhrase,
        ) -> Result<Arc<Mutex<dyn Identity>>>;
        fn create_admin_manager() -> AdminManager;
        fn get_master_identity(self: &AdminManager) -> Option<Arc<Mutex<dyn Identity>>>;
        fn request_verification_email(self: &mut AdminManager) -> Result<VoidType>;
        fn set_email(self: &mut AdminManager, email: String);
        fn verify_email(self: &mut AdminManager, verification_code: String) -> Result<VoidType>;

        type SeedPhrase;
        fn create_seed_phrase() -> Result<SeedPhrase>;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        type IdentityType;
        type SeedPhraseWords;
        fn get_identity_type(self: &Arc<Mutex<dyn Identity>>) -> IdentityType;
        fn get_name(self: &Arc<Mutex<dyn Identity>>) -> String;
        fn set_name(self: &mut Arc<Mutex<dyn Identity>>, name: String);
        fn get_pubkey(self: &Arc<Mutex<dyn Identity>>) -> Vec<u8>;
        fn get_fingerprint(self: &Arc<Mutex<dyn Identity>>) -> Vec<u8>;
        fn get_seed_phrase(self: &Arc<Mutex<dyn Identity>>) -> SeedPhraseWords;

        type VoidType;
        type ErrorType;
        fn to_string(self: &ErrorType) -> String;
        fn code(self: &ErrorType) -> u32;
    }
}
