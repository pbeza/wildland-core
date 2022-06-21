use crate::{
    admin_manager::AdminManager,
    api::{AdminManagerApi, AdminManagerError, IdentityPair, SeedPhrase, WildlandIdentityApi},
};
use ffi_macro::binding_wrapper;

// Define Error type and `()` type.
type ErrorType = AdminManagerError;
type VoidType = ();

#[binding_wrapper]
mod ffi_binding {
    use super::{AdminManagerApi, WildlandIdentityApi};
    extern "Rust" {
        type AdminManager;
        fn create_admin_manager() -> AdminManager;
        fn create_seed_phrase(self: &AdminManager) -> Result<SeedPhrase>;
        fn request_verification_email(self: &mut AdminManager) -> Result<VoidType>;
        fn set_email(self: &mut AdminManager, email: String);
        fn verify_email(self: &mut AdminManager, verification_code: String) -> Result<VoidType>;
        fn create_wildland_identities(
            self: &mut AdminManager,
            seed: &SeedPhrase,
            device_name: String,
        ) -> Result<IdentityPair>;

        type IdentityPair;
        fn forest_id(self: &IdentityPair) -> Arc<Mutex<dyn WildlandIdentityApi>>;
        fn device_id(self: &IdentityPair) -> Arc<Mutex<dyn WildlandIdentityApi>>;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        // type WildlandIdentityType;
        // fn get_identity_type(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> WildlandIdentityType;
        fn to_string(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> String;
        fn get_name(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> String;
        fn set_name(self: &mut Arc<Mutex<dyn WildlandIdentityApi>>, name: String);
        fn get_public_key(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> Vec<u8>;
        fn get_private_key(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> Vec<u8>;
        fn get_fingerprint_string(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> String;
        fn get_fingerprint(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> Vec<u8>;
        // fn save(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> Result<VoidType>;

        type VoidType;
        type ErrorType;
        fn to_string(self: &ErrorType) -> String;
        fn code(self: &ErrorType) -> u32;
    }
}
