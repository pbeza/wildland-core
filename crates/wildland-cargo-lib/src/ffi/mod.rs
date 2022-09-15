use crate::{
    api::user::UserApi, cargo_lib::CargoLib, create_cargo_lib, errors::*, MnemonicPayload,
    UserPayload,
};
use ffi_macro::binding_wrapper;
pub use wildland_corex::{
    CoreXError, CryptoError, ForestRetrievalError, LocalSecureStorage, LssError, LssResult,
    UserCreationError,
};

type VoidType = ();

pub type UserRetrievalExc = RetrievalError<ForestRetrievalError>;
pub type MnemonicCreationExc = CreationError<CryptoError>;
pub type UserCreationExc = CreationError<UserCreationError>;

type LssOptionalBytesResult = LssResult<Option<Vec<u8>>>;
fn new_ok_lss_optional_bytes(ok_val: OptionalBytes) -> LssOptionalBytesResult {
    Ok(ok_val)
}
fn new_err_lss_optional_bytes(err_val: String) -> LssOptionalBytesResult {
    Err(LssError(err_val))
}

type LssBoolResult = LssResult<bool>;
fn new_ok_lss_bool(ok_val: bool) -> LssBoolResult {
    Ok(ok_val)
}
fn new_err_lss_bool(err_val: String) -> LssBoolResult {
    Err(LssError(err_val))
}

type OptionalBytes = Option<Vec<u8>>;
fn new_some_bytes(bytes: Vec<u8>) -> OptionalBytes {
    Some(bytes)
}
fn new_none_bytes() -> OptionalBytes {
    None
}

type LssVecOfStringsResult = LssResult<Vec<String>>;
fn new_ok_lss_vec_of_strings(ok_val: Vec<String>) -> LssVecOfStringsResult {
    Ok(ok_val)
}
fn new_err_lss_vec_of_strings(err_val: String) -> LssVecOfStringsResult {
    Err(LssError(err_val))
}

type LssUsizeResult = LssResult<usize>;
fn new_ok_lss_usize(ok_val: usize) -> LssUsizeResult {
    Ok(ok_val)
}
fn new_err_lss_usize(err_val: String) -> LssUsizeResult {
    Err(LssError(err_val))
}

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
    enum MnemonicCreationExc {
        NotCreated(_),
    }
    enum UserCreationExc {
        NotCreated(_),
    }

    extern "Traits" {
        type LocalSecureStorage;
        fn insert(
            self: &dyn LocalSecureStorage,
            key: String,
            value: Vec<u8>,
        ) -> LssOptionalBytesResult;
        fn get(self: &dyn LocalSecureStorage, key: String) -> LssOptionalBytesResult;
        fn contains_key(self: &dyn LocalSecureStorage, key: String) -> LssBoolResult;
        fn keys(self: &dyn LocalSecureStorage) -> LssVecOfStringsResult;
        fn remove(self: &dyn LocalSecureStorage, key: String) -> LssOptionalBytesResult;
        fn len(self: &dyn LocalSecureStorage) -> LssUsizeResult;
        fn is_empty(self: &dyn LocalSecureStorage) -> LssBoolResult;
    }

    extern "Rust" {
        type VoidType;

        type LssOptionalBytesResult;
        fn new_ok_lss_optional_bytes(ok_val: OptionalBytes) -> LssOptionalBytesResult;
        fn new_err_lss_optional_bytes(err_val: String) -> LssOptionalBytesResult;
        type LssBoolResult;
        fn new_ok_lss_bool(ok_val: bool) -> LssBoolResult;
        fn new_err_lss_bool(err_val: String) -> LssBoolResult;
        type LssVecOfStringsResult;
        fn new_ok_lss_vec_of_strings(ok_val: Vec<String>) -> LssVecOfStringsResult;
        fn new_err_lss_vec_of_strings(err_val: String) -> LssVecOfStringsResult;
        type OptionalBytes;
        fn new_some_bytes(bytes: Vec<u8>) -> OptionalBytes;
        fn new_none_bytes() -> OptionalBytes;
        type LssUsizeResult;
        fn new_ok_lss_usize(ok_val: usize) -> LssUsizeResult;
        fn new_err_lss_usize(err_val: String) -> LssUsizeResult;

        fn create_cargo_lib(lss: &'static dyn LocalSecureStorage) -> CargoLib;
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
