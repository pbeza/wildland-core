//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::{Arc, Mutex};

use rusty_bind::binding_wrapper;
pub use wildland_corex::catlib_service::error::CatlibError;
use wildland_corex::entities::Identity;
use wildland_corex::{BridgeManifest, ContainerManifest, ForestManifest, Signers, StorageManifest};
pub use wildland_corex::{
    CoreXError,
    CryptoError,
    ForestRetrievalError,
    LocalSecureStorage,
    LssError,
    StorageTemplate,
};

use crate::api::cargo_lib::*;
use crate::api::cargo_user::*;
use crate::api::config::*;
use crate::api::foundation_storage::*;
use crate::api::user::*;
use crate::errors::storage::*;
use crate::errors::user::*;
use crate::errors::ExceptionTrait;

type VoidType = ();

#[cfg_attr(
    feature = "bindings",
    binding_wrapper(source = "../../_generated_ffi_code/interface.rs")
)]
mod ffi_binding {
    extern "ExceptionTrait" {
        fn reason(&self) -> String;
    }
    enum UserCreationError {
        UserAlreadyExists,
        MnemonicGenerationError(_),
        IdentityGenerationError(_),
        UserRetrievalError(_),
        ForestIdentityCreationError(_),
        LssError(_),
        EntropyTooLow,
        CatlibError(_),
    }
    enum UserRetrievalError {
        ForestRetrievalError(_),
        ForestNotFound(_),
        LssError(_),
        CatlibError(_),
        DeviceMetadataNotFound,
        UserNotFound,
    }
    enum FsaError {
        StorageAlreadyExists,
        EvsError(_),
        CryptoError(_),
        InvalidCredentialsFormat(_),
        LssError(_),
        CatlibError(_),
        StorageTemplateError(_),
        Generic(_),
    }
    enum LssError {
        Error(_),
    }
    enum ParseConfigError {
        Error(_),
    }
    enum CargoLibCreationError {
        Error(_),
    }
    enum CreateMnemonicError {
        InvalidMnemonicWords,
    }
    enum ForestMountError {
        Error,
    }
    enum CatlibError {
        NoRecordsFound(_),
        MalformedDatabaseEntry(_),
        RecordAlreadyExists(_),
        Generic(_),
    }
    enum ContainerMountError {
        Error,
    }
    enum ContainerUnmountError {
        Error,
    }
    enum FoundationCloudMode {
        Dev,
    }
    enum GetStorageTemplateError {
        LssError(_),
        DeserializationError(_),
    }

    extern "Traits" {

        // # traits required for logging configuration
        //
        fn get_use_logger(self: &dyn CargoCfgProvider) -> bool;
        fn get_log_level(self: &dyn CargoCfgProvider) -> String;
        fn get_log_use_ansi(self: &dyn CargoCfgProvider) -> bool;
        fn get_log_file_enabled(self: &dyn CargoCfgProvider) -> bool;
        fn get_log_file_path(self: &dyn CargoCfgProvider) -> Option<String>;
        fn get_log_file_rotate_directory(self: &dyn CargoCfgProvider) -> Option<String>;

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        fn get_oslog_category(self: &dyn CargoCfgProvider) -> Option<String>;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        fn get_oslog_subsystem(self: &dyn CargoCfgProvider) -> Option<String>;

        fn get_foundation_cloud_env_mode(self: &dyn CargoCfgProvider) -> FoundationCloudMode;

        // # traits required for lss:
        //
        fn insert(
            self: &dyn LocalSecureStorage,
            key: String,
            value: String,
        ) -> Result<Option<String>, LssError>;
        fn get(self: &dyn LocalSecureStorage, key: String) -> Result<Option<String>, LssError>;
        fn contains_key(self: &dyn LocalSecureStorage, key: String) -> Result<bool, LssError>;
        fn keys(self: &dyn LocalSecureStorage) -> Result<Vec<String>, LssError>;
        fn keys_starting_with(
            self: &dyn LocalSecureStorage,
            prefix: String,
        ) -> Result<Vec<String>, LssError>;
        fn remove(self: &dyn LocalSecureStorage, key: String) -> Result<Option<String>, LssError>;
        fn len(self: &dyn LocalSecureStorage) -> Result<usize, LssError>;
        fn is_empty(self: &dyn LocalSecureStorage) -> Result<bool, LssError>;
    }

    extern "Rust" {
        type VoidType;

        fn parse_config(raw_content: Vec<u8>) -> Result<CargoConfig, ParseConfigError>;
        fn collect_config(
            config_provider: &'static dyn CargoCfgProvider,
        ) -> Result<CargoConfig, ParseConfigError>;
        fn override_evs_url(self: &CargoConfig, new_evs_url: String);
        fn override_sc_url(self: &CargoConfig, new_sc_url: String);

        //
        // CargoLib
        //
        fn create_cargo_lib(
            lss: &'static dyn LocalSecureStorage,
            config: CargoConfig,
        ) -> Result<Arc<Mutex<CargoLib>>, CargoLibCreationError>;
        fn user_api(self: &Arc<Mutex<CargoLib>>) -> UserApi;

        //
        // UserApi
        //

        // Mnemonic
        fn generate_mnemonic(self: &UserApi) -> Result<MnemonicPayload, CreateMnemonicError>;
        fn create_mnemonic_from_vec(
            self: &UserApi,
            words: Vec<String>,
        ) -> Result<MnemonicPayload, CreateMnemonicError>;
        fn create_mnemonic_from_string(
            self: &UserApi,
            mnemonic_str: String,
        ) -> Result<MnemonicPayload, CreateMnemonicError>;

        // User
        fn create_user_from_entropy(
            self: &UserApi,
            entropy: Vec<u8>,
            device_name: String,
        ) -> Result<CargoUser, UserCreationError>;
        fn create_user_from_mnemonic(
            self: &UserApi,
            mnemonic: &MnemonicPayload,
            device_name: String,
        ) -> Result<CargoUser, UserCreationError>;
        fn get_user(self: &UserApi) -> Result<CargoUser, UserRetrievalError>;

        //
        // MnemonicPayload
        //
        fn stringify(self: &MnemonicPayload) -> String;
        fn get_vec(self: &MnemonicPayload) -> Vec<String>;

        type Identity;
        type Signers;

        //
        // CargoUser
        //
        fn stringify(self: &CargoUser) -> String;
        fn get_containers(
            self: &CargoUser,
        ) -> Result<Vec<Arc<Mutex<dyn ContainerManifest>>>, CatlibError>;
        fn create_container(
            self: &CargoUser,
            name: String,
            storage_templates: &StorageTemplate,
            path: String,
        ) -> Result<Arc<Mutex<dyn ContainerManifest>>, CatlibError>;
        fn get_storage_templates(
            self: &CargoUser,
        ) -> Result<Vec<StorageTemplate>, GetStorageTemplateError>;

        // Foundation Storage
        type FreeTierProcessHandle;
        fn request_free_tier_storage(
            self: &CargoUser,
            email: String,
        ) -> Result<FreeTierProcessHandle, FsaError>;
        fn verify_email(
            self: &CargoUser,
            process_handle: &FreeTierProcessHandle,
            verification_token: String,
        ) -> Result<StorageTemplate, FsaError>;
        fn is_free_storage_granted(self: &CargoUser) -> Result<bool, CatlibError>;

        //
        // ForestManifest
        //
        fn add_signer(
            self: &Arc<Mutex<dyn ForestManifest>>,
            signer: Identity,
        ) -> Result<bool, CatlibError>;
        fn del_signer(
            self: &Arc<Mutex<dyn ForestManifest>>,
            signer: Identity,
        ) -> Result<bool, CatlibError>;
        fn containers(
            self: &Arc<Mutex<dyn ForestManifest>>,
        ) -> Result<Vec<Arc<Mutex<dyn ContainerManifest>>>, CatlibError>;
        fn update(
            self: &Arc<Mutex<dyn ForestManifest>>,
            data: Vec<u8>,
        ) -> Result<VoidType, CatlibError>;
        fn remove(self: &Arc<Mutex<dyn ForestManifest>>) -> Result<bool, CatlibError>;
        fn create_container(
            self: &Arc<Mutex<dyn ForestManifest>>,
            name: String,
            storage_data: &StorageTemplate,
            path: String,
        ) -> Result<Arc<Mutex<dyn ContainerManifest>>, CatlibError>;
        fn create_bridge(
            self: &Arc<Mutex<dyn ForestManifest>>,
            path: String,
            link_data: Vec<u8>,
        ) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError>;
        fn find_bridge(
            self: &Arc<Mutex<dyn ForestManifest>>,
            path: String,
        ) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError>;
        fn find_containers(
            self: &Arc<Mutex<dyn ForestManifest>>,
            paths: Vec<String>,
            include_subdirs: bool,
        ) -> Result<Vec<Arc<Mutex<dyn ContainerManifest>>>, CatlibError>;
        fn owner(self: &Arc<Mutex<dyn ForestManifest>>) -> Identity;
        fn signers(self: &Arc<Mutex<dyn ForestManifest>>) -> Result<Signers, CatlibError>;

        //
        // ContainerManifest
        //
        fn get_storages(
            self: &Arc<Mutex<dyn ContainerManifest>>,
        ) -> Result<Vec<Arc<Mutex<dyn StorageManifest>>>, CatlibError>;
        fn add_storage(
            self: &Arc<Mutex<dyn ContainerManifest>>,
            templates: &StorageTemplate,
        ) -> Result<Arc<Mutex<dyn StorageManifest>>, CatlibError>;
        fn add_path(
            self: &Arc<Mutex<dyn ContainerManifest>>,
            path: String,
        ) -> Result<bool, CatlibError>;
        fn delete_path(
            self: &Arc<Mutex<dyn ContainerManifest>>,
            path: String,
        ) -> Result<bool, CatlibError>;
        fn get_paths(self: &Arc<Mutex<dyn ContainerManifest>>) -> Result<Vec<String>, CatlibError>;
        fn set_name(
            self: &Arc<Mutex<dyn ContainerManifest>>,
            new_name: String,
        ) -> Result<VoidType, CatlibError>;
        fn remove(self: &Arc<Mutex<dyn ContainerManifest>>) -> Result<VoidType, CatlibError>;
        fn forest(
            self: &Arc<Mutex<dyn ContainerManifest>>,
        ) -> Result<Arc<Mutex<dyn ForestManifest>>, CatlibError>;
        fn name(self: &Arc<Mutex<dyn ContainerManifest>>) -> Result<String, CatlibError>;
        fn stringify(self: &Arc<Mutex<dyn ContainerManifest>>) -> String;

        //
        // StorageManifest
        //
        fn container(
            self: &Arc<Mutex<dyn StorageManifest>>,
        ) -> Result<Arc<Mutex<dyn ContainerManifest>>, CatlibError>;
        fn update(
            self: &Arc<Mutex<dyn StorageManifest>>,
            data: Vec<u8>,
        ) -> Result<VoidType, CatlibError>;
        fn remove(self: &Arc<Mutex<dyn StorageManifest>>) -> Result<bool, CatlibError>;
        fn data(self: &Arc<Mutex<dyn StorageManifest>>) -> Result<Vec<u8>, CatlibError>;

        //
        // BridgeManifets
        //
        fn forest(
            self: &Arc<Mutex<dyn BridgeManifest>>,
        ) -> Result<Arc<Mutex<dyn ForestManifest>>, CatlibError>;
        fn update(
            self: &Arc<Mutex<dyn BridgeManifest>>,
            link: Vec<u8>,
        ) -> Result<VoidType, CatlibError>;
        fn remove(self: &Arc<Mutex<dyn BridgeManifest>>) -> Result<bool, CatlibError>;
        fn path(self: &Arc<Mutex<dyn BridgeManifest>>) -> Result<String, CatlibError>;

        //
        // Storage
        //
        fn stringify(self: &StorageTemplate) -> String;
    }
}
