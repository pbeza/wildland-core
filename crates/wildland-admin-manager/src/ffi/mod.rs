use crate::{
    admin_manager::AdminManager,
    api::{
        AdminManagerApi, AdminManagerError, IdentityPair, SeedPhrase, WildlandIdentity,
        WildlandIdentityType,
    },
};
use ffi_macro::binding_wrapper;
use std::sync::{Arc, Mutex};

// Define Error type and `()` type.
type ErrorType = AdminManagerError;
type VoidType = ();

#[binding_wrapper]
mod ffi_binding {
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
        fn forest_id(self: &IdentityPair) -> Arc<Mutex<WildlandIdentity>>;
        fn device_id(self: &IdentityPair) -> Arc<Mutex<WildlandIdentity>>;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        type WildlandIdentityType;
        // TODO WILX-95 generate code for handling enums with binding_wrapper macro
        fn is_same(self: &WildlandIdentityType, other: &WildlandIdentityType) -> bool;
        fn is_forest(self: &WildlandIdentityType) -> bool;
        fn is_device(self: &WildlandIdentityType) -> bool;

        fn get_type(self: &Arc<Mutex<WildlandIdentity>>) -> WildlandIdentityType;
        fn to_string(self: &Arc<Mutex<WildlandIdentity>>) -> String;
        fn get_name(self: &Arc<Mutex<WildlandIdentity>>) -> String;
        fn set_name(self: &mut Arc<Mutex<WildlandIdentity>>, name: String);
        fn get_public_key(self: &Arc<Mutex<WildlandIdentity>>) -> Vec<u8>;
        fn get_private_key(self: &Arc<Mutex<WildlandIdentity>>) -> Vec<u8>;
        fn get_fingerprint_string(self: &Arc<Mutex<WildlandIdentity>>) -> String;
        fn get_fingerprint(self: &Arc<Mutex<WildlandIdentity>>) -> Vec<u8>;
        fn save(self: &Arc<Mutex<WildlandIdentity>>) -> Result<VoidType>;

        type VoidType;
        type ErrorType;
        fn to_string(self: &ErrorType) -> String;
        fn code(self: &ErrorType) -> u32;
    }
}
