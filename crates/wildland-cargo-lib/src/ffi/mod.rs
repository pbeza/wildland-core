use crate::{
    api::user::UserApi, cargo_lib::CargoLib, create_cargo_lib, error::*, MnemonicPayload,
    UserPayload,
};
use ffi_macro::binding_wrapper;
pub use wildland_corex::{CoreXError, CryptoError, ForestRetrievalError};

type VoidType = ();

pub type UserRetrievalError = RetrievalError<ForestRetrievalError>;
pub type MnemonicCreationError = CreationError<CryptoError>;
pub type CargoLibCreationError = CreationError<String>;
pub type UserCreationError = CreationError<wildland_corex::UserCreationError>;

#[binding_wrapper]
mod ffi_binding {
    enum WildlandXDomain {
        CoreX,
    }
    extern "ExceptionTrait" {
        fn reason(&self) -> String;
        fn domain(&self) -> WildlandXDomain;
    }
    enum UserRetrievalError {
        NotFound(String),
        Unexpected(VoidType), // In fact in Rust this variant keeps ForestRetrievalError.
                              // However client code does not need to know anything about ForestRetrievalError
                              // because it generates exceptions only basing on variants names.
                              // Inner field is needed only in Rust code in ExceptionTrait impl block,
                              // so here it is possible to avoid specifying ForestRetrievalError type and use VoidType instead.
    }
    enum CargoLibCreationError {
        NotCreated(String),
    }
    enum MnemonicCreationError {
        NotCreated(VoidType), // The same as above: used VoidType instead CryptoError
    }
    enum UserCreationError {
        NotCreated(VoidType), // The same as above: used VoidType instead wildland_corex::UserCreationError
    }

    extern "Rust" {
        type VoidType;

        fn create_cargo_lib(lss_path: String) -> Result<CargoLib, CargoLibCreationError>;
        fn user_api(self: &CargoLib) -> UserApi;

        fn generate_mnemonic(self: &UserApi) -> Result<MnemonicPayload, MnemonicCreationError>;
        fn create_user_from_entropy(
            self: &UserApi,
            entropy: Vec<u8>,
            device_name: String,
        ) -> Result<VoidType, UserCreationError>;
        fn create_user_from_mnemonic(
            self: &UserApi,
            mnemonic: &MnemonicPayload,
            device_name: String,
        ) -> Result<VoidType, UserCreationError>;
        fn get_user(self: &UserApi) -> Result<UserPayload, UserRetrievalError>;

        fn get_string(self: &MnemonicPayload) -> String;
        fn get_vec(self: &MnemonicPayload) -> Vec<String>;

        fn get_string(self: &UserPayload) -> String;
    }
}
