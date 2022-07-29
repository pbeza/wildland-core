use crate::{
    admin_manager::AdminManager, api::user::UserApi, create_admin_manager, AdminManagerError,
    MnemonicPayload, UserPayload
};
use ffi_macro::binding_wrapper;

// Define Error type and `()` type.
type ErrorType = AdminManagerError;
type VoidType = ();

#[binding_wrapper]
mod ffi_binding {

    extern "Rust" {
        type AdminManager;
        fn create_admin_manager(lss_path: String) -> Result<AdminManager>;
        fn user_api(self: &AdminManager) -> &UserApi;

        type UserApi;
        fn generate_mnemonic(self: &UserApi) -> Result<MnemonicPayload>;
        fn create_user_from_entropy(
            self: &UserApi,
            entropy: Vec<u8>,
            device_name: String,
        ) -> Result<VoidType>;
        fn create_user_from_mnemonic(
            self: &UserApi,
            mnemonic: &MnemonicPayload,
            device_name: String,
        ) -> Result<VoidType>;
        fn get_user(self: &UserApi) -> Result<Option<UserPayload>>;

        type MnemonicPayload;
        fn get_string(self: &MnemonicPayload) -> String;
        fn get_vec(self: &MnemonicPayload) -> Vec<String>;

        type UserPayload;

        type VoidType;
        type ErrorType;
        fn to_string(self: &ErrorType) -> String;
        fn code(self: &ErrorType) -> u32;
    }
}
