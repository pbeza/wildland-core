use crate::{
    api::user::UserApi, cargo_lib::CargoLib, create_cargo_lib, CargoLibError, MnemonicPayload,
};
use ffi_macro::binding_wrapper;

// Define Error type and `()` type.
type ErrorType = CargoLibError;
type VoidType = ();

#[binding_wrapper]
mod ffi_binding {
    extern "Rust" {
        type CargoLib;
        fn create_cargo_lib(lss_path: String) -> Result<CargoLib>;
        fn user_api(self: &CargoLib) -> &UserApi;

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
        fn get_user(self: &UserApi) -> VoidType;

        type MnemonicPayload;
        fn get_string(self: &MnemonicPayload) -> String;
        fn get_vec(self: &MnemonicPayload) -> Vec<String>;

        type VoidType;
        type ErrorType;
        fn to_string(self: &ErrorType) -> String;
        fn code(self: &ErrorType) -> u32;
    }
}
