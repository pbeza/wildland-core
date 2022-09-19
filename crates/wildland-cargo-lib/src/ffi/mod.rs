use crate::{
    api::{config::CargoCfgProvider, user::UserApi},
    cargo_lib::CargoLib,
    create_cargo_lib,
    errors::*,
    MnemonicPayload, UserPayload,
};
use ffi_macro::binding_wrapper;
pub use wildland_corex::{
    CoreXError, CryptoError, ForestRetrievalError, LssError, UserCreationError,
};

type VoidType = ();

pub type UserRetrievalExc = RetrievalError<ForestRetrievalError>;
pub type MnemonicCreationExc = CreationError<CryptoError>;
pub type CargoLibCreationExc = CreationError<LssError>;
pub type UserCreationExc = CreationError<UserCreationError>;

#[binding_wrapper]
mod ffi_binding {
    enum WildlandXDomain {
        CargoUser,
        Crypto,
        Catlib,
        CoreX,
        Dfs,
    }
    extern "ExceptionTrait" {
        fn reason(&self) -> String;
        fn domain(&self) -> WildlandXDomain;
    }
    enum UserRetrievalExc {
        NotFound(_),
        Unexpected(_),
    }
    enum CargoLibCreationExc {
        NotCreated(_),
    }
    enum MnemonicCreationExc {
        NotCreated(_),
    }
    enum UserCreationExc {
        NotCreated(_),
    }

    extern "Traits" {
        type CargoCfgProvider;
        /// Returns configuration in json form
        fn get_config(self: &dyn CargoCfgProvider) -> Vec<u8>;
    }

    extern "Rust" {
        type VoidType;

        fn create_cargo_lib(lss_path: String) -> Result<CargoLib, CargoLibCreationExc>;
        fn user_api(self: &CargoLib) -> UserApi;

        fn generate_mnemonic(self: &UserApi) -> Result<MnemonicPayload, MnemonicCreationExc>;
        fn create_user_from_entropy(
            self: &UserApi,
            entropy: Vec<u8>,
            device_name: String,
        ) -> Result<VoidType, UserCreationExc>;
        fn create_user_from_mnemonic(
            self: &UserApi,
            mnemonic: &MnemonicPayload,
            device_name: String,
        ) -> Result<VoidType, UserCreationExc>;
        fn get_user(self: &UserApi) -> Result<UserPayload, UserRetrievalExc>;

        fn get_string(self: &MnemonicPayload) -> String;
        fn get_vec(self: &MnemonicPayload) -> Vec<String>;

        fn get_string(self: &UserPayload) -> String;
    }
}
