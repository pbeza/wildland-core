use crate::{
    admin_manager::AdminManager, api::user::UserApi, create_admin_manager, AdminManagerError,
    SeedPhrase,
};
use ffi_macro::binding_wrapper;

// Define Error type and `()` type.
type ErrorType = AdminManagerError;
type VoidType = ();

#[binding_wrapper]
mod ffi_binding {
    extern "Rust" {
        type AdminManager;
        fn create_admin_manager() -> AdminManager;
        fn user_api(self: &AdminManager) -> &UserApi;

        type UserApi;
        fn generate_mnemonic(self: &UserApi) -> Result<SeedPhrase>;
        fn create_user_from_entropy(self: &UserApi, entropy: Vec<u8>) -> VoidType;
        fn create_user_from_mnemonic(self: &UserApi, mnemonic: SeedPhrase) -> VoidType;
        fn get_user(self: &UserApi) -> VoidType;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        type VoidType;
        type ErrorType;
        fn to_string(self: &ErrorType) -> String;
        fn code(self: &ErrorType) -> u32;
    }
}
