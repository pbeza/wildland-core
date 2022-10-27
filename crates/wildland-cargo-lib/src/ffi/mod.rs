//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
    api::{
        cargo_lib::*, cargo_user::*, config::*, container::*, foundation_storage::*, storage::*,
        storage_template::*, user::*,
    },
    errors::{
        container::*, retrieval_error::*, single_variant::*, storage::*, user::*, ExceptionTrait,
    },
};
use rusty_bind::binding_wrapper;
use std::sync::{Arc, Mutex};
pub use wildland_corex::{
    CatlibError, CoreXError, CryptoError, ForestRetrievalError, LocalSecureStorage, LssError,
    LssResult,
};

type VoidType = ();

pub type UserRetrievalExc = RetrievalError<UserRetrievalError>;
pub type MnemonicCreationExc = SingleVariantError<CryptoError>;
pub type StringExc = SingleVariantError<String>; // Used for simple errors originating inside CargoLib (not in dependant modules)
pub type UserCreationExc = SingleVariantError<UserCreationError>;
pub type CargoLibCreationExc = SingleVariantError<CargoLibCreationError>;
pub type ConfigParseExc = SingleVariantError<ParseConfigError>;
pub type FsaExc = FsaError;
pub type CatlibExc = SingleVariantError<CatlibError>;
pub type ContainerMountExc = SingleVariantError<ContainerMountError>;
pub type ContainerUnmountExc = SingleVariantError<ContainerUnmountError>;
pub type AddStorageExc = SingleVariantError<AddStorageError>;
pub type DeleteStorageExc = SingleVariantError<DeleteStorageError>;
pub type GetStoragesExc = SingleVariantError<GetStoragesError>;
pub type ForestMountExc = SingleVariantError<ForestMountError>;
pub type ContainerDeletionExc = ContainerDeletionError;

pub type LssOptionalBytesResult = LssResult<Option<Vec<u8>>>;
/// constructor of `LssResult<Option<Vec<u8>>>` (aka [`LssOptionalBytesResult`]) with Ok variant
pub fn new_ok_lss_optional_bytes(ok_val: OptionalBytes) -> LssOptionalBytesResult {
    Ok(ok_val)
}
/// constructor of `LssResult<Option<Vec<u8>>>` (aka [`LssOptionalBytesResult`]) with Err variant
pub fn new_err_lss_optional_bytes(err_val: String) -> LssOptionalBytesResult {
    Err(LssError(err_val))
}

pub type LssBoolResult = LssResult<bool>;
/// constructor of `LssResult<bool>` (aka [`LssBoolResult`]) with Ok variant
pub fn new_ok_lss_bool(ok_val: bool) -> LssBoolResult {
    Ok(ok_val)
}
/// constructor of `LssResult<bool>` (aka [`LssBoolResult`]) with Err variant
pub fn new_err_lss_bool(err_val: String) -> LssBoolResult {
    Err(LssError(err_val))
}

pub type OptionalBytes = Option<Vec<u8>>;
/// constructor of `Option<Vec<u8>>` (aka [`OptionalBytes`]) with Some value
pub fn new_some_bytes(bytes: Vec<u8>) -> OptionalBytes {
    Some(bytes)
}
/// constructor of `Option<Vec<u8>>` (aka [`OptionalBytes`]) with None value
pub fn new_none_bytes() -> OptionalBytes {
    None
}

pub type OptionalString = Option<String>;
/// constructor of `Option<String>` (aka [`OptionalString`]) with Some value
fn new_some_string(s: String) -> OptionalString {
    Some(s)
}
/// constructor of `Option<String>` (aka [`OptionalString`]) with None value
fn new_none_string() -> OptionalString {
    None
}

pub type LssVecOfStringsResult = LssResult<Vec<String>>;
/// constructor of `LssResult<Vec<String>>` (aka [`LssVecOfStringsResult`]) with Ok variant
fn new_ok_lss_vec_of_strings(ok_val: Vec<String>) -> LssVecOfStringsResult {
    Ok(ok_val)
}
/// constructor of `LssResult<Vec<String>>` (aka [`LssVecOfStringsResult`]) with Err variant
fn new_err_lss_vec_of_strings(err_val: String) -> LssVecOfStringsResult {
    Err(LssError(err_val))
}

pub type LssUsizeResult = LssResult<usize>;
/// constructor of `LssResult<usize>` (aka [`LssUsizeResult`]) with Ok variant
fn new_ok_lss_usize(ok_val: usize) -> LssUsizeResult {
    Ok(ok_val)
}
/// constructor of `LssResult<usize>` (aka [`LssUsizeResult`]) with Err variant
pub fn new_err_lss_usize(err_val: String) -> LssUsizeResult {
    Err(LssError(err_val))
}

#[binding_wrapper]
mod ffi_binding {
    extern "ExceptionTrait" {
        fn reason(&self) -> String;
    }
    enum UserRetrievalExc {
        NotFound(_),
        Unexpected(_),
    }
    enum MnemonicCreationExc {
        Failure(_),
    }
    enum UserCreationExc {
        Failure(_),
    }
    enum CargoLibCreationExc {
        Failure(_),
    }
    enum ConfigParseExc {
        Failure(_),
    }
    enum FsaExc {
        StorageAlreadyExists,
        EvsError(_),
        CryptoError(_),
        InvalidCredentialsFormat(_),
    }
    enum StringExc {
        Failure(_),
    }
    enum ForestMountExc {
        Failure(_),
    }
    enum CatlibExc {
        Failure(_),
    }
    enum ContainerMountExc {
        Failure(_),
    }
    enum ContainerUnmountExc {
        Failure(_),
    }
    enum GetStoragesExc {
        Failure(_),
    }
    enum DeleteStorageExc {
        Failure(_),
    }
    enum AddStorageExc {
        Failure(_),
    }
    enum ContainerDeletionExc {
        CatlibError(_),
        ContainerDataLockError(_),
    }

    extern "Traits" {
        fn get_log_level(self: &dyn CargoCfgProvider) -> String;
        fn get_log_file(self: &dyn CargoCfgProvider) -> OptionalString;
        fn get_evs_url(self: &dyn CargoCfgProvider) -> String;
        fn get_sc_url(self: &dyn CargoCfgProvider) -> String;

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
        type LssUsizeResult;
        fn new_ok_lss_usize(ok_val: usize) -> LssUsizeResult;
        fn new_err_lss_usize(err_val: String) -> LssUsizeResult;

        type OptionalBytes;
        fn new_some_bytes(bytes: Vec<u8>) -> OptionalBytes;
        fn new_none_bytes() -> OptionalBytes;
        type OptionalString;
        fn new_some_string(s: String) -> OptionalString;
        fn new_none_string() -> OptionalString;

        type CargoConfig;
        fn parse_config(raw_content: Vec<u8>) -> Result<CargoConfig, ConfigParseExc>;
        fn collect_config(
            config_provider: &'static dyn CargoCfgProvider,
        ) -> Result<CargoConfig, ConfigParseExc>;

        fn create_cargo_lib(
            lss: &'static dyn LocalSecureStorage,
            config: CargoConfig,
        ) -> Result<Arc<Mutex<CargoLib>>, CargoLibCreationExc>;
        fn user_api(self: &Arc<Mutex<CargoLib>>) -> UserApi;
        fn foundation_storage_api(self: &Arc<Mutex<CargoLib>>) -> FoundationStorageApi;

        fn request_free_tier_storage(
            self: &FoundationStorageApi,
            email: String,
        ) -> Result<FreeTierProcessHandle, FsaExc>;
        fn verify_email(
            self: &FoundationStorageApi,
            process_handle: &FreeTierProcessHandle,
            verification_token: String,
        ) -> Result<StorageTemplate, FsaExc>;
        type FreeTierProcessHandle;

        fn stringify(self: &StorageTemplate) -> String;

        fn generate_mnemonic(self: &UserApi) -> Result<MnemonicPayload, MnemonicCreationExc>;
        fn create_mnemonic_from_vec(
            self: &UserApi,
            words: Vec<String>,
        ) -> Result<MnemonicPayload, StringExc>;
        fn create_user_from_entropy(
            self: &UserApi,
            entropy: Vec<u8>,
            device_name: String,
        ) -> Result<CargoUser, UserCreationExc>;
        fn create_user_from_mnemonic(
            self: &UserApi,
            mnemonic: &MnemonicPayload,
            device_name: String,
        ) -> Result<CargoUser, UserCreationExc>;
        fn get_user(self: &UserApi) -> Result<CargoUser, UserRetrievalExc>;

        fn stringify(self: &MnemonicPayload) -> String;
        fn get_vec(self: &MnemonicPayload) -> Vec<String>;

        fn stringify(self: &CargoUser) -> String;
        fn mount_forest(self: &CargoUser) -> Result<VoidType, ForestMountExc>;
        fn get_containers(
            self: &CargoUser,
            include_unmounted: bool,
        ) -> Result<Vec<Container>, CatlibExc>;
        fn create_container(
            self: &CargoUser,
            name: String,
            storage_templates: &StorageTemplate,
        ) -> Result<Container, CatlibExc>;
        fn delete_container(
            self: &CargoUser,
            container: &Container,
        ) -> Result<VoidType, ContainerDeletionExc>;

        fn mount(self: &Container) -> Result<VoidType, ContainerMountExc>;
        fn unmount(self: &Container) -> Result<VoidType, ContainerUnmountExc>;
        fn is_mounted(self: &Container) -> bool;
        fn get_storages(self: &Container) -> Result<Vec<Storage>, GetStoragesExc>;
        fn delete_storage(
            self: &Container,
            storage: &Storage,
        ) -> Result<VoidType, DeleteStorageExc>;
        fn add_storage(
            self: &Container,
            templates: &StorageTemplate,
        ) -> Result<VoidType, AddStorageExc>;
        fn set_name(self: &Container, new_name: String);
        fn stringify(self: &Container) -> String;
        fn duplicate(self: &Container) -> Result<Container, CatlibExc>;

        fn stringify(self: &Storage) -> String;
    }
}
