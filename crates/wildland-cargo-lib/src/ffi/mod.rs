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
pub use wildland_corex::dfs::interface::*;
use wildland_corex::entities::Identity;
use wildland_corex::{BridgeManifest, Signers, StorageManifest};
pub use wildland_corex::{
    Container,
    ContainerManagerError,
    CoreXError,
    CryptoError,
    ForestRetrievalError,
    LocalSecureStorage,
    LssError,
    StorageTemplate,
    StorageTemplateError,
};

use crate::api::cargo_lib::*;
use crate::api::cargo_user::*;
use crate::api::config::*;
use crate::api::foundation_storage::*;
use crate::api::user::*;
use crate::errors::storage::*;
use crate::errors::user::*;
use crate::errors::ExceptionTrait;

mod wrapper {
    use wildland_corex::{StorageTemplate, StorageTemplateError};

    use super::WlPermissions;

    pub(crate) fn storage_template_from_json(
        content: Vec<u8>,
    ) -> Result<StorageTemplate, StorageTemplateError> {
        StorageTemplate::from_json(content)
    }

    pub(crate) fn storage_template_from_yaml(
        content: Vec<u8>,
    ) -> Result<StorageTemplate, StorageTemplateError> {
        StorageTemplate::from_yaml(content)
    }

    pub(crate) fn readonly_wl_permissions() -> WlPermissions {
        WlPermissions::readonly()
    }

    pub(crate) fn read_write_wl_permissions() -> WlPermissions {
        WlPermissions::read_write()
    }
}

use self::wrapper::{
    read_write_wl_permissions,
    readonly_wl_permissions,
    storage_template_from_json,
    storage_template_from_yaml,
};

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
        Generic(_),
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
        Generic(_),
    }
    enum FsaError {
        EvsError(_),
        CryptoError(_),
        InvalidCredentialsFormat(_),
        LssError(_),
        CatlibError(_),
        StorageTemplateError(_),
        Generic(_),
    }
    enum StorageTemplateError {
        SerdeErr(_),
        TemplateEngineErr(_),
    }
    enum LssError {
        Error(_),
    }
    enum ParseConfigError {
        Error(_),
    }
    enum CreateMnemonicError {
        InvalidMnemonicWords,
    }
    enum CatlibError {
        NoRecordsFound(_),
        MalformedDatabaseEntry(_),
        RecordAlreadyExists(_),
        Generic(_),
    }
    enum ContainerManagerError {
        AlreadyMounted,
        MountingError(_),
        ContainerNotMounted,
    }
    enum FoundationCloudMode {
        Dev,
    }
    enum GetStorageTemplateError {
        CatlibError(_),
        DeserializationError(_),
    }
    enum DfsFrontendError {
        NotAFile,
        NotADirectory,
        NoSuchPath,
        PathResolutionError(_),
        Generic(_),
        FileAlreadyClosed,
        PathAlreadyExists,
        ParentDoesNotExist,
        StorageNotResponsive,
        ReadOnlyPath,
        DirNotEmpty,
        SeekError,
        ConcurrentIssue,
    }
    enum NodeType {
        File,
        Dir,
        Symlink,
        Other,
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
        ) -> Arc<Mutex<CargoLib>>;
        fn user_api(self: &Arc<Mutex<CargoLib>>) -> UserApi;
        fn dfs_api(self: &Arc<Mutex<CargoLib>>) -> Arc<Mutex<dyn DfsFrontend>>;

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
        fn get_containers(self: &CargoUser) -> Result<Vec<Container>, CatlibError>;
        fn create_container(
            self: &CargoUser,
            name: String,
            storage_templates: &StorageTemplate,
            path: String,
        ) -> Result<Container, CatlibError>;
        fn get_storage_templates(
            self: &CargoUser,
        ) -> Result<Vec<StorageTemplate>, GetStorageTemplateError>;
        fn save_storage_template(
            self: &CargoUser,
            tpl: &StorageTemplate,
        ) -> Result<String, CatlibError>;
        fn mount(
            self: &CargoUser,
            container: &Container,
        ) -> Result<VoidType, ContainerManagerError>;
        fn unmount(
            self: &CargoUser,
            container: &Container,
        ) -> Result<VoidType, ContainerManagerError>;

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
        // Container
        //
        fn get_storages(
            self: &Container,
        ) -> Result<Vec<Arc<Mutex<dyn StorageManifest>>>, CatlibError>;
        fn add_storage(
            self: &Container,
            templates: &StorageTemplate,
        ) -> Result<Arc<Mutex<dyn StorageManifest>>, CatlibError>;
        fn add_path(self: &Container, path: String) -> Result<bool, CatlibError>;
        fn delete_path(self: &Container, path: String) -> Result<bool, CatlibError>;
        fn get_paths(self: &Container) -> Result<Vec<String>, CatlibError>;
        fn set_name(self: &Container, new_name: String) -> Result<VoidType, CatlibError>;
        fn remove(self: &Container) -> Result<VoidType, CatlibError>;
        fn name(self: &Container) -> Result<String, CatlibError>;
        fn stringify(self: &Container) -> String;

        //
        // StorageManifest
        //
        fn update(
            self: &Arc<Mutex<dyn StorageManifest>>,
            data: Vec<u8>,
        ) -> Result<VoidType, CatlibError>;
        fn remove(self: &Arc<Mutex<dyn StorageManifest>>) -> Result<bool, CatlibError>;
        fn data(self: &Arc<Mutex<dyn StorageManifest>>) -> Result<Vec<u8>, CatlibError>;

        //
        // BridgeManifets
        //
        fn update(
            self: &Arc<Mutex<dyn BridgeManifest>>,
            link: Vec<u8>,
        ) -> Result<VoidType, CatlibError>;
        fn remove(self: &Arc<Mutex<dyn BridgeManifest>>) -> Result<bool, CatlibError>;
        fn path(self: &Arc<Mutex<dyn BridgeManifest>>) -> Result<String, CatlibError>;

        //
        // StorageTemplate
        //
        fn storage_template_from_json(
            content: Vec<u8>,
        ) -> Result<StorageTemplate, StorageTemplateError>;
        fn storage_template_from_yaml(
            content: Vec<u8>,
        ) -> Result<StorageTemplate, StorageTemplateError>;

        fn to_json(self: &StorageTemplate) -> Result<String, StorageTemplateError>;
        fn to_yaml(self: &StorageTemplate) -> Result<String, StorageTemplateError>;

        fn set_name(self: &StorageTemplate, name: String) -> VoidType;

        fn uuid_str(self: &StorageTemplate) -> String;
        fn backend_type(self: &StorageTemplate) -> String;
        fn name(self: &StorageTemplate) -> Option<String>;

        fn stringify(self: &StorageTemplate) -> String;

        //
        // DFS Frontend
        //
        fn read_dir(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            path: String,
        ) -> Result<Vec<String>, DfsFrontendError>;
        fn metadata(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            path: String,
        ) -> Result<Stat, DfsFrontendError>;
        fn open(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            path: String,
        ) -> Result<FileHandle, DfsFrontendError>;
        fn close(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file_handle: &FileHandle,
        ) -> Result<VoidType, DfsFrontendError>;
        fn create_file(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            path: String,
        ) -> Result<FileHandle, DfsFrontendError>;
        fn remove_file(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            path: String,
        ) -> Result<VoidType, DfsFrontendError>;
        fn rename(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            old_path: String,
            new_path: String,
        ) -> Result<VoidType, DfsFrontendError>;
        fn set_permissions(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            path: String,
            permissions: WlPermissions,
        ) -> Result<VoidType, DfsFrontendError>;
        fn set_file_permissions(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
            permissions: WlPermissions,
        ) -> Result<VoidType, DfsFrontendError>;
        fn set_owner(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            path: String,
        ) -> Result<VoidType, DfsFrontendError>;
        fn create_dir(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            requested_path: String,
        ) -> Result<VoidType, DfsFrontendError>;
        fn remove_dir(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            requested_path: String,
        ) -> Result<VoidType, DfsFrontendError>;
        fn read(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
            count: usize,
        ) -> Result<Vec<u8>, DfsFrontendError>;
        fn write(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
            buf: Vec<u8>,
        ) -> Result<usize, DfsFrontendError>;
        fn set_length(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
            length: usize,
        ) -> Result<VoidType, DfsFrontendError>;
        fn sync(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
        ) -> Result<VoidType, DfsFrontendError>;
        fn set_times(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
            access_time: Option<UnixTimestamp>,
            modification_time: Option<UnixTimestamp>,
        ) -> Result<VoidType, DfsFrontendError>;
        fn file_metadata(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
        ) -> Result<Stat, DfsFrontendError>;
        fn file_stat_fs(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
        ) -> Result<FsStat, DfsFrontendError>;
        fn seek_from_start(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
            pos_from_start: u64,
        ) -> Result<usize, DfsFrontendError>;
        fn seek_from_current(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
            pos_from_current: i64,
        ) -> Result<usize, DfsFrontendError>;
        fn seek_from_end(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            file: &FileHandle,
            pos_from_end: i64,
        ) -> Result<usize, DfsFrontendError>;
        fn sync_all(self: &Arc<Mutex<dyn DfsFrontend>>) -> Result<VoidType, DfsFrontendError>;
        fn stat_fs(
            self: &Arc<Mutex<dyn DfsFrontend>>,
            path: String,
        ) -> Result<FsStat, DfsFrontendError>;

        //
        // FileHandle
        //
        type FileHandle;

        //
        // Stat
        //
        fn node_type(self: &Stat) -> NodeType;
        fn size(self: &Stat) -> usize;
        fn access_time(self: &Stat) -> Option<UnixTimestamp>;
        fn modification_time(self: &Stat) -> Option<UnixTimestamp>;
        fn change_time(self: &Stat) -> Option<UnixTimestamp>;

        //
        // UnixTimestamp
        //
        fn sec(self: &UnixTimestamp) -> u64;
        fn nano_sec(self: &UnixTimestamp) -> u32;

        //
        // FsStat
        //
        fn block_size(self: &FsStat) -> u64;
        fn io_size(self: &FsStat) -> Option<u64>;
        fn blocks(self: &FsStat) -> u64;
        fn free_blocks(self: &FsStat) -> u64;
        fn blocks_available(self: &FsStat) -> u64;
        fn nodes(self: &FsStat) -> u64;
        fn name_length(self: &FsStat) -> u64;

        //
        // WlPermissions
        //
        fn is_readonly(self: &WlPermissions) -> bool;
        fn set_readonly(self: &WlPermissions, readonly: bool);
        fn readonly_wl_permissions() -> WlPermissions;
        fn read_write_wl_permissions() -> WlPermissions;
    }
}
