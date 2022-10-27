#[cfg(feature = "bindings")]
pub mod ffi {
    use crate::{
        api::{
            cargo_lib::*, cargo_user::*, config::*, container::*, foundation_storage::*,
            storage::*, storage_template::*, user::*,
        },
        errors::{
            container::*, retrieval_error::*, single_variant::*, storage::*, user::*,
            ExceptionTrait,
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
    pub type StringExc = SingleVariantError<String>;
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
    mod ffi_binding {
        use super::*;
        pub struct LocalSecureStorage(std::sync::atomic::AtomicPtr<*mut std::ffi::c_void>);
        impl super::LocalSecureStorage for LocalSecureStorage {
            fn insert(&self, key: String, value: Vec<u8>) -> LssOptionalBytesResult {
                unsafe {
                    *Box::from_raw(__rustybind__LocalSecureStorage_insert(
                        self,
                        Box::into_raw(Box::new(key)),
                        Box::into_raw(Box::new(value)),
                    ))
                }
            }
            fn get(&self, key: String) -> LssOptionalBytesResult {
                unsafe {
                    *Box::from_raw(__rustybind__LocalSecureStorage_get(
                        self,
                        Box::into_raw(Box::new(key)),
                    ))
                }
            }
            fn contains_key(&self, key: String) -> LssBoolResult {
                unsafe {
                    *Box::from_raw(__rustybind__LocalSecureStorage_contains_key(
                        self,
                        Box::into_raw(Box::new(key)),
                    ))
                }
            }
            fn keys(&self) -> LssVecOfStringsResult {
                unsafe { *Box::from_raw(__rustybind__LocalSecureStorage_keys(self)) }
            }
            fn remove(&self, key: String) -> LssOptionalBytesResult {
                unsafe {
                    *Box::from_raw(__rustybind__LocalSecureStorage_remove(
                        self,
                        Box::into_raw(Box::new(key)),
                    ))
                }
            }
            fn len(&self) -> LssUsizeResult {
                unsafe { *Box::from_raw(__rustybind__LocalSecureStorage_len(self)) }
            }
            fn is_empty(&self) -> LssBoolResult {
                unsafe { *Box::from_raw(__rustybind__LocalSecureStorage_is_empty(self)) }
            }
        }
        extern "C" {
            #[link_name = "__rustybind__$LocalSecureStorage$insert"]
            fn __rustybind__LocalSecureStorage_insert(
                _self: *const LocalSecureStorage,
                key: *mut String,
                value: *mut Vec<u8>,
            ) -> *mut LssOptionalBytesResult;
        }
        extern "C" {
            #[link_name = "__rustybind__$LocalSecureStorage$get"]
            fn __rustybind__LocalSecureStorage_get(
                _self: *const LocalSecureStorage,
                key: *mut String,
            ) -> *mut LssOptionalBytesResult;
        }
        extern "C" {
            #[link_name = "__rustybind__$LocalSecureStorage$contains_key"]
            fn __rustybind__LocalSecureStorage_contains_key(
                _self: *const LocalSecureStorage,
                key: *mut String,
            ) -> *mut LssBoolResult;
        }
        extern "C" {
            #[link_name = "__rustybind__$LocalSecureStorage$keys"]
            fn __rustybind__LocalSecureStorage_keys(
                _self: *const LocalSecureStorage,
            ) -> *mut LssVecOfStringsResult;
        }
        extern "C" {
            #[link_name = "__rustybind__$LocalSecureStorage$remove"]
            fn __rustybind__LocalSecureStorage_remove(
                _self: *const LocalSecureStorage,
                key: *mut String,
            ) -> *mut LssOptionalBytesResult;
        }
        extern "C" {
            #[link_name = "__rustybind__$LocalSecureStorage$len"]
            fn __rustybind__LocalSecureStorage_len(
                _self: *const LocalSecureStorage,
            ) -> *mut LssUsizeResult;
        }
        extern "C" {
            #[link_name = "__rustybind__$LocalSecureStorage$is_empty"]
            fn __rustybind__LocalSecureStorage_is_empty(
                _self: *const LocalSecureStorage,
            ) -> *mut LssBoolResult;
        }
        pub struct CargoCfgProvider(std::sync::atomic::AtomicPtr<*mut std::ffi::c_void>);
        impl super::CargoCfgProvider for CargoCfgProvider {
            fn get_log_level(&self) -> String {
                unsafe { *Box::from_raw(__rustybind__CargoCfgProvider_get_log_level(self)) }
            }
            fn get_log_file(&self) -> OptionalString {
                unsafe { *Box::from_raw(__rustybind__CargoCfgProvider_get_log_file(self)) }
            }
            fn get_evs_url(&self) -> String {
                unsafe { *Box::from_raw(__rustybind__CargoCfgProvider_get_evs_url(self)) }
            }
            fn get_sc_url(&self) -> String {
                unsafe { *Box::from_raw(__rustybind__CargoCfgProvider_get_sc_url(self)) }
            }
        }
        extern "C" {
            #[link_name = "__rustybind__$CargoCfgProvider$get_log_level"]
            fn __rustybind__CargoCfgProvider_get_log_level(
                _self: *const CargoCfgProvider,
            ) -> *mut String;
        }
        extern "C" {
            #[link_name = "__rustybind__$CargoCfgProvider$get_log_file"]
            fn __rustybind__CargoCfgProvider_get_log_file(
                _self: *const CargoCfgProvider,
            ) -> *mut OptionalString;
        }
        extern "C" {
            #[link_name = "__rustybind__$CargoCfgProvider$get_evs_url"]
            fn __rustybind__CargoCfgProvider_get_evs_url(
                _self: *const CargoCfgProvider,
            ) -> *mut String;
        }
        extern "C" {
            #[link_name = "__rustybind__$CargoCfgProvider$get_sc_url"]
            fn __rustybind__CargoCfgProvider_get_sc_url(
                _self: *const CargoCfgProvider,
            ) -> *mut String;
        }
        type FreeTierProcessHandleResultWithFsaExc = Result<FreeTierProcessHandle, FsaExc>;
        type VoidTypeResultWithCatlibExc = Result<VoidType, CatlibExc>;
        type VoidTypeResultWithContainerMountExc = Result<VoidType, ContainerMountExc>;
        type MnemonicPayloadResultWithMnemonicCreationExc =
            Result<MnemonicPayload, MnemonicCreationExc>;
        type SharedMutexCargoLibResultWithCargoLibCreationExc =
            Result<Arc<Mutex<CargoLib>>, CargoLibCreationExc>;
        type VecContainerResultWithCatlibExc = Result<Vec<Container>, CatlibExc>;
        type CargoConfigResultWithConfigParseExc = Result<CargoConfig, ConfigParseExc>;
        type MnemonicPayloadResultWithStringExc = Result<MnemonicPayload, StringExc>;
        type VoidTypeResultWithAddStorageExc = Result<VoidType, AddStorageExc>;
        type VoidTypeResultWithContainerUnmountExc = Result<VoidType, ContainerUnmountExc>;
        type VoidTypeResultWithForestMountExc = Result<VoidType, ForestMountExc>;
        type StorageTemplateResultWithFsaExc = Result<StorageTemplate, FsaExc>;
        type ContainerResultWithCatlibExc = Result<Container, CatlibExc>;
        type VoidTypeResultWithDeleteStorageExc = Result<VoidType, DeleteStorageExc>;
        type CargoUserResultWithUserRetrievalExc = Result<CargoUser, UserRetrievalExc>;
        type VecStorageResultWithGetStoragesExc = Result<Vec<Storage>, GetStoragesExc>;
        type CargoUserResultWithUserCreationExc = Result<CargoUser, UserCreationExc>;
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayload$stringify"]
            pub extern "C" fn stringify(_self: *mut MnemonicPayload) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&mut *_self).stringify() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayload$get_vec"]
            pub extern "C" fn get_vec(_self: *mut MnemonicPayload) -> *mut Vec<String> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).get_vec() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUser$stringify"]
            pub extern "C" fn stringify(_self: *mut CargoUser) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&mut *_self).stringify() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUser$mount_forest"]
            pub extern "C" fn mount_forest(
                _self: *mut CargoUser,
            ) -> *mut Result<VoidType, ForestMountExc> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).mount_forest() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUser$get_containers"]
            pub extern "C" fn get_containers(
                _self: *mut CargoUser,
                include_unmounted: bool,
            ) -> *mut Result<Vec<Container>, CatlibExc> {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).get_containers(include_unmounted)
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUser$create_container"]
            pub extern "C" fn create_container(
                _self: *mut CargoUser,
                name: *mut String,
                storage_templates: *mut StorageTemplate,
            ) -> *mut Result<Container, CatlibExc> {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).create_container(*Box::from_raw(name), &*storage_templates)
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUser$delete_container"]
            pub extern "C" fn delete_container(
                _self: *mut CargoUser,
                container: *mut Container,
            ) -> *mut Result<VoidType, CatlibExc> {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).delete_container(&*container)
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Storage$stringify"]
            pub extern "C" fn stringify(_self: *mut Storage) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&mut *_self).stringify() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FoundationStorageApi$request_free_tier_storage"]
            pub extern "C" fn request_free_tier_storage(
                _self: *mut FoundationStorageApi,
                email: *mut String,
            ) -> *mut Result<FreeTierProcessHandle, FsaExc> {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).request_free_tier_storage(*Box::from_raw(email))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FoundationStorageApi$verify_email"]
            pub extern "C" fn verify_email(
                _self: *mut FoundationStorageApi,
                process_handle: *mut FreeTierProcessHandle,
                verification_token: *mut String,
            ) -> *mut Result<StorageTemplate, FsaExc> {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).verify_email(&*process_handle, *Box::from_raw(verification_token))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$SharedMutexCargoLib$user_api"]
            pub extern "C" fn user_api(_self: *mut Arc<Mutex<CargoLib>>) -> *mut UserApi {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).lock().unwrap().user_api()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$SharedMutexCargoLib$foundation_storage_api"]
            pub extern "C" fn foundation_storage_api(
                _self: *mut Arc<Mutex<CargoLib>>,
            ) -> *mut FoundationStorageApi {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).lock().unwrap().foundation_storage_api()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StorageTemplate$stringify"]
            pub extern "C" fn stringify(_self: *mut StorageTemplate) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&mut *_self).stringify() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserApi$generate_mnemonic"]
            pub extern "C" fn generate_mnemonic(
                _self: *mut UserApi,
            ) -> *mut Result<MnemonicPayload, MnemonicCreationExc> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).generate_mnemonic() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserApi$create_mnemonic_from_vec"]
            pub extern "C" fn create_mnemonic_from_vec(
                _self: *mut UserApi,
                words: *mut Vec<String>,
            ) -> *mut Result<MnemonicPayload, StringExc> {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).create_mnemonic_from_vec(*Box::from_raw(words))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserApi$create_user_from_entropy"]
            pub extern "C" fn create_user_from_entropy(
                _self: *mut UserApi,
                entropy: *mut Vec<u8>,
                device_name: *mut String,
            ) -> *mut Result<CargoUser, UserCreationExc> {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).create_user_from_entropy(
                        *Box::from_raw(entropy),
                        *Box::from_raw(device_name),
                    )
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserApi$create_user_from_mnemonic"]
            pub extern "C" fn create_user_from_mnemonic(
                _self: *mut UserApi,
                mnemonic: *mut MnemonicPayload,
                device_name: *mut String,
            ) -> *mut Result<CargoUser, UserCreationExc> {
                Box::into_raw(Box::new(unsafe {
                    (&mut *_self).create_user_from_mnemonic(&*mnemonic, *Box::from_raw(device_name))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserApi$get_user"]
            pub extern "C" fn get_user(
                _self: *mut UserApi,
            ) -> *mut Result<CargoUser, UserRetrievalExc> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).get_user() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$mount"]
            pub extern "C" fn mount(
                _self: *mut Container,
            ) -> *mut Result<VoidType, ContainerMountExc> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).mount() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$unmount"]
            pub extern "C" fn unmount(
                _self: *mut Container,
            ) -> *mut Result<VoidType, ContainerUnmountExc> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).unmount() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$is_mounted"]
            pub extern "C" fn is_mounted(_self: *mut Container) -> bool {
                unsafe { (&mut *_self).is_mounted() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$get_storages"]
            pub extern "C" fn get_storages(
                _self: *mut Container,
            ) -> *mut Result<Vec<Storage>, GetStoragesExc> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).get_storages() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$delete_storage"]
            pub extern "C" fn delete_storage(
                _self: *mut Container,
                storage: *mut Storage,
            ) -> *mut Result<VoidType, DeleteStorageExc> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).delete_storage(&*storage) }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$add_storage"]
            pub extern "C" fn add_storage(
                _self: *mut Container,
                templates: *mut StorageTemplate,
            ) -> *mut Result<VoidType, AddStorageExc> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).add_storage(&*templates) }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$set_name"]
            pub extern "C" fn set_name(_self: *mut Container, new_name: *mut String) -> () {
                unsafe { (&mut *_self).set_name(*Box::from_raw(new_name)) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$stringify"]
            pub extern "C" fn stringify(_self: *mut Container) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&mut *_self).stringify() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$duplicate"]
            pub extern "C" fn duplicate(
                _self: *mut Container,
            ) -> *mut Result<Container, CatlibExc> {
                Box::into_raw(Box::new(unsafe { (&mut *_self).duplicate() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FsaExc$reason"]
            pub extern "C" fn reason(_self: *const FsaExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$GetStoragesExc$reason"]
            pub extern "C" fn reason(_self: *const GetStoragesExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerUnmountExc$reason"]
            pub extern "C" fn reason(_self: *const ContainerUnmountExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CatlibExc$reason"]
            pub extern "C" fn reason(_self: *const CatlibExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoLibCreationExc$reason"]
            pub extern "C" fn reason(_self: *const CargoLibCreationExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ForestMountExc$reason"]
            pub extern "C" fn reason(_self: *const ForestMountExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$DeleteStorageExc$reason"]
            pub extern "C" fn reason(_self: *const DeleteStorageExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$AddStorageExc$reason"]
            pub extern "C" fn reason(_self: *const AddStorageExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ConfigParseExc$reason"]
            pub extern "C" fn reason(_self: *const ConfigParseExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerMountExc$reason"]
            pub extern "C" fn reason(_self: *const ContainerMountExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserRetrievalExc$reason"]
            pub extern "C" fn reason(_self: *const UserRetrievalExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserCreationExc$reason"]
            pub extern "C" fn reason(_self: *const UserCreationExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StringExc$reason"]
            pub extern "C" fn reason(_self: *const StringExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicCreationExc$reason"]
            pub extern "C" fn reason(_self: *const MnemonicCreationExc) -> *mut String {
                Box::into_raw(Box::new(unsafe { (&*_self).reason() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_ok_lss_optional_bytes"]
            pub extern "C" fn new_ok_lss_optional_bytes(
                ok_val: *mut OptionalBytes,
            ) -> *mut LssOptionalBytesResult {
                Box::into_raw(Box::new(unsafe {
                    super::new_ok_lss_optional_bytes(*Box::from_raw(ok_val))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_err_lss_optional_bytes"]
            pub extern "C" fn new_err_lss_optional_bytes(
                err_val: *mut String,
            ) -> *mut LssOptionalBytesResult {
                Box::into_raw(Box::new(unsafe {
                    super::new_err_lss_optional_bytes(*Box::from_raw(err_val))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_ok_lss_bool"]
            pub extern "C" fn new_ok_lss_bool(ok_val: bool) -> *mut LssBoolResult {
                Box::into_raw(Box::new(unsafe { super::new_ok_lss_bool(ok_val) }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_err_lss_bool"]
            pub extern "C" fn new_err_lss_bool(err_val: *mut String) -> *mut LssBoolResult {
                Box::into_raw(Box::new(unsafe {
                    super::new_err_lss_bool(*Box::from_raw(err_val))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_ok_lss_vec_of_strings"]
            pub extern "C" fn new_ok_lss_vec_of_strings(
                ok_val: *mut Vec<String>,
            ) -> *mut LssVecOfStringsResult {
                Box::into_raw(Box::new(unsafe {
                    super::new_ok_lss_vec_of_strings(*Box::from_raw(ok_val))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_err_lss_vec_of_strings"]
            pub extern "C" fn new_err_lss_vec_of_strings(
                err_val: *mut String,
            ) -> *mut LssVecOfStringsResult {
                Box::into_raw(Box::new(unsafe {
                    super::new_err_lss_vec_of_strings(*Box::from_raw(err_val))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_ok_lss_usize"]
            pub extern "C" fn new_ok_lss_usize(ok_val: usize) -> *mut LssUsizeResult {
                Box::into_raw(Box::new(unsafe { super::new_ok_lss_usize(ok_val) }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_err_lss_usize"]
            pub extern "C" fn new_err_lss_usize(err_val: *mut String) -> *mut LssUsizeResult {
                Box::into_raw(Box::new(unsafe {
                    super::new_err_lss_usize(*Box::from_raw(err_val))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_some_bytes"]
            pub extern "C" fn new_some_bytes(bytes: *mut Vec<u8>) -> *mut OptionalBytes {
                Box::into_raw(Box::new(unsafe {
                    super::new_some_bytes(*Box::from_raw(bytes))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_none_bytes"]
            pub extern "C" fn new_none_bytes() -> *mut OptionalBytes {
                Box::into_raw(Box::new(unsafe { super::new_none_bytes() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_some_string"]
            pub extern "C" fn new_some_string(s: *mut String) -> *mut OptionalString {
                Box::into_raw(Box::new(unsafe {
                    super::new_some_string(*Box::from_raw(s))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$new_none_string"]
            pub extern "C" fn new_none_string() -> *mut OptionalString {
                Box::into_raw(Box::new(unsafe { super::new_none_string() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$parse_config"]
            pub extern "C" fn parse_config(
                raw_content: *mut Vec<u8>,
            ) -> *mut Result<CargoConfig, ConfigParseExc> {
                Box::into_raw(Box::new(unsafe {
                    super::parse_config(*Box::from_raw(raw_content))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$collect_config"]
            pub extern "C" fn collect_config(
                config_provider: *mut CargoCfgProvider,
            ) -> *mut Result<CargoConfig, ConfigParseExc> {
                Box::into_raw(Box::new(unsafe {
                    super::collect_config(&*config_provider)
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$create_cargo_lib"]
            pub extern "C" fn create_cargo_lib(
                lss: *mut LocalSecureStorage,
                config: *mut CargoConfig,
            ) -> *mut Result<Arc<Mutex<CargoLib>>, CargoLibCreationExc> {
                Box::into_raw(Box::new(unsafe {
                    super::create_cargo_lib(&*lss, *Box::from_raw(config))
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FreeTierProcessHandleResultWithFsaExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut FreeTierProcessHandleResultWithFsaExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FreeTierProcessHandleResultWithFsaExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut FreeTierProcessHandleResultWithFsaExc,
            ) -> *mut FreeTierProcessHandle {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FreeTierProcessHandleResultWithFsaExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut FreeTierProcessHandleResultWithFsaExc,
            ) -> *mut FsaExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithCatlibExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut VoidTypeResultWithCatlibExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithCatlibExc$unwrap"]
            pub extern "C" fn unwrap(_self: *mut VoidTypeResultWithCatlibExc) -> *mut VoidType {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithCatlibExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut VoidTypeResultWithCatlibExc,
            ) -> *mut CatlibExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerMountExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut VoidTypeResultWithContainerMountExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerMountExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut VoidTypeResultWithContainerMountExc,
            ) -> *mut VoidType {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerMountExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut VoidTypeResultWithContainerMountExc,
            ) -> *mut ContainerMountExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithMnemonicCreationExc$is_ok"]
            pub extern "C" fn is_ok(
                _self: *mut MnemonicPayloadResultWithMnemonicCreationExc,
            ) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithMnemonicCreationExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut MnemonicPayloadResultWithMnemonicCreationExc,
            ) -> *mut MnemonicPayload {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithMnemonicCreationExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut MnemonicPayloadResultWithMnemonicCreationExc,
            ) -> *mut MnemonicCreationExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$SharedMutexCargoLibResultWithCargoLibCreationExc$is_ok"]
            pub extern "C" fn is_ok(
                _self: *mut SharedMutexCargoLibResultWithCargoLibCreationExc,
            ) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$SharedMutexCargoLibResultWithCargoLibCreationExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut SharedMutexCargoLibResultWithCargoLibCreationExc,
            ) -> *mut Arc<Mutex<CargoLib>> {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$SharedMutexCargoLibResultWithCargoLibCreationExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut SharedMutexCargoLibResultWithCargoLibCreationExc,
            ) -> *mut CargoLibCreationExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainerResultWithCatlibExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut VecContainerResultWithCatlibExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainerResultWithCatlibExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut VecContainerResultWithCatlibExc,
            ) -> *mut Vec<Container> {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainerResultWithCatlibExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut VecContainerResultWithCatlibExc,
            ) -> *mut CatlibExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoConfigResultWithConfigParseExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut CargoConfigResultWithConfigParseExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoConfigResultWithConfigParseExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut CargoConfigResultWithConfigParseExc,
            ) -> *mut CargoConfig {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoConfigResultWithConfigParseExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut CargoConfigResultWithConfigParseExc,
            ) -> *mut ConfigParseExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithStringExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut MnemonicPayloadResultWithStringExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithStringExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut MnemonicPayloadResultWithStringExc,
            ) -> *mut MnemonicPayload {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithStringExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut MnemonicPayloadResultWithStringExc,
            ) -> *mut StringExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithAddStorageExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut VoidTypeResultWithAddStorageExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithAddStorageExc$unwrap"]
            pub extern "C" fn unwrap(_self: *mut VoidTypeResultWithAddStorageExc) -> *mut VoidType {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithAddStorageExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut VoidTypeResultWithAddStorageExc,
            ) -> *mut AddStorageExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerUnmountExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut VoidTypeResultWithContainerUnmountExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerUnmountExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut VoidTypeResultWithContainerUnmountExc,
            ) -> *mut VoidType {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerUnmountExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut VoidTypeResultWithContainerUnmountExc,
            ) -> *mut ContainerUnmountExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithForestMountExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut VoidTypeResultWithForestMountExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithForestMountExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut VoidTypeResultWithForestMountExc,
            ) -> *mut VoidType {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithForestMountExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut VoidTypeResultWithForestMountExc,
            ) -> *mut ForestMountExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StorageTemplateResultWithFsaExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut StorageTemplateResultWithFsaExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StorageTemplateResultWithFsaExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut StorageTemplateResultWithFsaExc,
            ) -> *mut StorageTemplate {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StorageTemplateResultWithFsaExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut StorageTemplateResultWithFsaExc,
            ) -> *mut FsaExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerResultWithCatlibExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut ContainerResultWithCatlibExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerResultWithCatlibExc$unwrap"]
            pub extern "C" fn unwrap(_self: *mut ContainerResultWithCatlibExc) -> *mut Container {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerResultWithCatlibExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut ContainerResultWithCatlibExc,
            ) -> *mut CatlibExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithDeleteStorageExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut VoidTypeResultWithDeleteStorageExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithDeleteStorageExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut VoidTypeResultWithDeleteStorageExc,
            ) -> *mut VoidType {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithDeleteStorageExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut VoidTypeResultWithDeleteStorageExc,
            ) -> *mut DeleteStorageExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserRetrievalExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut CargoUserResultWithUserRetrievalExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserRetrievalExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut CargoUserResultWithUserRetrievalExc,
            ) -> *mut CargoUser {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserRetrievalExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut CargoUserResultWithUserRetrievalExc,
            ) -> *mut UserRetrievalExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorageResultWithGetStoragesExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut VecStorageResultWithGetStoragesExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorageResultWithGetStoragesExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut VecStorageResultWithGetStoragesExc,
            ) -> *mut Vec<Storage> {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorageResultWithGetStoragesExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut VecStorageResultWithGetStoragesExc,
            ) -> *mut GetStoragesExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserCreationExc$is_ok"]
            pub extern "C" fn is_ok(_self: *mut CargoUserResultWithUserCreationExc) -> bool {
                unsafe { (&mut *_self).is_ok() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserCreationExc$unwrap"]
            pub extern "C" fn unwrap(
                _self: *mut CargoUserResultWithUserCreationExc,
            ) -> *mut CargoUser {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserCreationExc$unwrap_err_unchecked"]
            pub extern "C" fn unwrap_err_unchecked(
                _self: *mut CargoUserResultWithUserCreationExc,
            ) -> *mut UserCreationExc {
                Box::into_raw(Box::new(unsafe {
                    Box::from_raw(_self).unwrap_err_unchecked()
                }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalStorage$from"]
            pub extern "C" fn from(val: *mut Storage) -> *mut Option<Storage> {
                Box::into_raw(Box::new(unsafe { Option::from(*Box::from_raw(val)) }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalStorage$default"]
            pub extern "C" fn default() -> *mut Option<Storage> {
                Box::into_raw(Box::new(unsafe { Option::default() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalStorage$is_some"]
            pub extern "C" fn is_some(_self: *mut Option<Storage>) -> bool {
                unsafe { (&mut *_self).is_some() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalStorage$unwrap"]
            pub extern "C" fn unwrap(_self: *mut Option<Storage>) -> *mut Storage {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Optionalu8$from"]
            pub extern "C" fn from(val: u8) -> *mut Option<u8> {
                Box::into_raw(Box::new(unsafe { Option::from(val) }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Optionalu8$default"]
            pub extern "C" fn default() -> *mut Option<u8> {
                Box::into_raw(Box::new(unsafe { Option::default() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Optionalu8$is_some"]
            pub extern "C" fn is_some(_self: *mut Option<u8>) -> bool {
                unsafe { (&mut *_self).is_some() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Optionalu8$unwrap"]
            pub extern "C" fn unwrap(_self: *mut Option<u8>) -> u8 {
                unsafe { Box::from_raw(_self).unwrap() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorage$new"]
            pub extern "C" fn new() -> *mut Vec<Storage> {
                Box::into_raw(Box::new(unsafe { Vec::new() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorage$get"]
            pub extern "C" fn get(_self: *mut Vec<Storage>, index: usize) -> *mut Option<Storage> {
                unsafe { Box::into_raw(Box::new(unsafe { (&*_self).get(index).cloned() })) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorage$push"]
            pub extern "C" fn push(_self: *mut Vec<Storage>, obj: *mut Storage) -> () {
                unsafe { (&mut *_self).push(*Box::from_raw(obj)) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorage$len"]
            pub extern "C" fn len(_self: *mut Vec<Storage>) -> usize {
                unsafe { (&mut *_self).len() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Vecu8$new"]
            pub extern "C" fn new() -> *mut Vec<u8> {
                Box::into_raw(Box::new(unsafe { Vec::new() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Vecu8$get"]
            pub extern "C" fn get(_self: *mut Vec<u8>, index: usize) -> *mut Option<u8> {
                unsafe { Box::into_raw(Box::new(unsafe { (&*_self).get(index).cloned() })) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Vecu8$push"]
            pub extern "C" fn push(_self: *mut Vec<u8>, obj: u8) -> () {
                unsafe { (&mut *_self).push(obj) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Vecu8$len"]
            pub extern "C" fn len(_self: *mut Vec<u8>) -> usize {
                unsafe { (&mut *_self).len() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainer$new"]
            pub extern "C" fn new() -> *mut Vec<Container> {
                Box::into_raw(Box::new(unsafe { Vec::new() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainer$get"]
            pub extern "C" fn get(
                _self: *mut Vec<Container>,
                index: usize,
            ) -> *mut Option<Container> {
                unsafe { Box::into_raw(Box::new(unsafe { (&*_self).get(index).cloned() })) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainer$push"]
            pub extern "C" fn push(_self: *mut Vec<Container>, obj: *mut Container) -> () {
                unsafe { (&mut *_self).push(*Box::from_raw(obj)) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainer$len"]
            pub extern "C" fn len(_self: *mut Vec<Container>) -> usize {
                unsafe { (&mut *_self).len() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalRustString$from"]
            pub extern "C" fn from(val: *mut String) -> *mut Option<String> {
                Box::into_raw(Box::new(unsafe { Option::from(*Box::from_raw(val)) }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalRustString$default"]
            pub extern "C" fn default() -> *mut Option<String> {
                Box::into_raw(Box::new(unsafe { Option::default() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalRustString$is_some"]
            pub extern "C" fn is_some(_self: *mut Option<String>) -> bool {
                unsafe { (&mut *_self).is_some() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalRustString$unwrap"]
            pub extern "C" fn unwrap(_self: *mut Option<String>) -> *mut String {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecRustString$new"]
            pub extern "C" fn new() -> *mut Vec<String> {
                Box::into_raw(Box::new(unsafe { Vec::new() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecRustString$get"]
            pub extern "C" fn get(_self: *mut Vec<String>, index: usize) -> *mut Option<String> {
                unsafe { Box::into_raw(Box::new(unsafe { (&*_self).get(index).cloned() })) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecRustString$push"]
            pub extern "C" fn push(_self: *mut Vec<String>, obj: *mut String) -> () {
                unsafe { (&mut *_self).push(*Box::from_raw(obj)) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecRustString$len"]
            pub extern "C" fn len(_self: *mut Vec<String>) -> usize {
                unsafe { (&mut *_self).len() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalContainer$from"]
            pub extern "C" fn from(val: *mut Container) -> *mut Option<Container> {
                Box::into_raw(Box::new(unsafe { Option::from(*Box::from_raw(val)) }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalContainer$default"]
            pub extern "C" fn default() -> *mut Option<Container> {
                Box::into_raw(Box::new(unsafe { Option::default() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalContainer$is_some"]
            pub extern "C" fn is_some(_self: *mut Option<Container>) -> bool {
                unsafe { (&mut *_self).is_some() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalContainer$unwrap"]
            pub extern "C" fn unwrap(_self: *mut Option<Container>) -> *mut Container {
                Box::into_raw(Box::new(unsafe { Box::from_raw(_self).unwrap() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserRetrievalExc$clone"]
            pub extern "C" fn clone(_self: *mut UserRetrievalExc) -> *mut UserRetrievalExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserRetrievalExc$drop"]
            pub extern "C" fn drop(_self: *mut UserRetrievalExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserCreationExc$clone"]
            pub extern "C" fn clone(_self: *mut UserCreationExc) -> *mut UserCreationExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserCreationExc$drop"]
            pub extern "C" fn drop(_self: *mut UserCreationExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CatlibExc$clone"]
            pub extern "C" fn clone(_self: *mut CatlibExc) -> *mut CatlibExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CatlibExc$drop"]
            pub extern "C" fn drop(_self: *mut CatlibExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayload$clone"]
            pub extern "C" fn clone(_self: *mut MnemonicPayload) -> *mut MnemonicPayload {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayload$drop"]
            pub extern "C" fn drop(_self: *mut MnemonicPayload) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalStorage$clone"]
            pub extern "C" fn clone(_self: *mut Option<Storage>) -> *mut Option<Storage> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalStorage$drop"]
            pub extern "C" fn drop(_self: *mut Option<Storage>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUser$clone"]
            pub extern "C" fn clone(_self: *mut CargoUser) -> *mut CargoUser {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUser$drop"]
            pub extern "C" fn drop(_self: *mut CargoUser) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FreeTierProcessHandle$clone"]
            pub extern "C" fn clone(
                _self: *mut FreeTierProcessHandle,
            ) -> *mut FreeTierProcessHandle {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FreeTierProcessHandle$drop"]
            pub extern "C" fn drop(_self: *mut FreeTierProcessHandle) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$LssOptionalBytesResult$clone"]
            pub extern "C" fn clone(
                _self: *mut LssOptionalBytesResult,
            ) -> *mut LssOptionalBytesResult {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$LssOptionalBytesResult$drop"]
            pub extern "C" fn drop(_self: *mut LssOptionalBytesResult) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Optionalu8$clone"]
            pub extern "C" fn clone(_self: *mut Option<u8>) -> *mut Option<u8> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Optionalu8$drop"]
            pub extern "C" fn drop(_self: *mut Option<u8>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FreeTierProcessHandleResultWithFsaExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<FreeTierProcessHandle, FsaExc>,
            ) -> *mut Result<FreeTierProcessHandle, FsaExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FreeTierProcessHandleResultWithFsaExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<FreeTierProcessHandle, FsaExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$SharedMutexCargoLib$clone"]
            pub extern "C" fn clone(_self: *mut Arc<Mutex<CargoLib>>) -> *mut Arc<Mutex<CargoLib>> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$SharedMutexCargoLib$drop"]
            pub extern "C" fn drop(_self: *mut Arc<Mutex<CargoLib>>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FsaExc$clone"]
            pub extern "C" fn clone(_self: *mut FsaExc) -> *mut FsaExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FsaExc$drop"]
            pub extern "C" fn drop(_self: *mut FsaExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StringExc$clone"]
            pub extern "C" fn clone(_self: *mut StringExc) -> *mut StringExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StringExc$drop"]
            pub extern "C" fn drop(_self: *mut StringExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidType$clone"]
            pub extern "C" fn clone(_self: *mut VoidType) -> *mut VoidType {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidType$drop"]
            pub extern "C" fn drop(_self: *mut VoidType) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithCatlibExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<VoidType, CatlibExc>,
            ) -> *mut Result<VoidType, CatlibExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithCatlibExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<VoidType, CatlibExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerMountExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<VoidType, ContainerMountExc>,
            ) -> *mut Result<VoidType, ContainerMountExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerMountExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<VoidType, ContainerMountExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ForestMountExc$clone"]
            pub extern "C" fn clone(_self: *mut ForestMountExc) -> *mut ForestMountExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ForestMountExc$drop"]
            pub extern "C" fn drop(_self: *mut ForestMountExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorage$clone"]
            pub extern "C" fn clone(_self: *mut Vec<Storage>) -> *mut Vec<Storage> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorage$drop"]
            pub extern "C" fn drop(_self: *mut Vec<Storage>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithMnemonicCreationExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<MnemonicPayload, MnemonicCreationExc>,
            ) -> *mut Result<MnemonicPayload, MnemonicCreationExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithMnemonicCreationExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<MnemonicPayload, MnemonicCreationExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$SharedMutexCargoLibResultWithCargoLibCreationExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<Arc<Mutex<CargoLib>>, CargoLibCreationExc>,
            ) -> *mut Result<Arc<Mutex<CargoLib>>, CargoLibCreationExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$SharedMutexCargoLibResultWithCargoLibCreationExc$drop"]
            pub extern "C" fn drop(
                _self: *mut Result<Arc<Mutex<CargoLib>>, CargoLibCreationExc>,
            ) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainerResultWithCatlibExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<Vec<Container>, CatlibExc>,
            ) -> *mut Result<Vec<Container>, CatlibExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainerResultWithCatlibExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<Vec<Container>, CatlibExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoLibCreationExc$clone"]
            pub extern "C" fn clone(_self: *mut CargoLibCreationExc) -> *mut CargoLibCreationExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoLibCreationExc$drop"]
            pub extern "C" fn drop(_self: *mut CargoLibCreationExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$AddStorageExc$clone"]
            pub extern "C" fn clone(_self: *mut AddStorageExc) -> *mut AddStorageExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$AddStorageExc$drop"]
            pub extern "C" fn drop(_self: *mut AddStorageExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoConfigResultWithConfigParseExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<CargoConfig, ConfigParseExc>,
            ) -> *mut Result<CargoConfig, ConfigParseExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoConfigResultWithConfigParseExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<CargoConfig, ConfigParseExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$LssVecOfStringsResult$clone"]
            pub extern "C" fn clone(
                _self: *mut LssVecOfStringsResult,
            ) -> *mut LssVecOfStringsResult {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$LssVecOfStringsResult$drop"]
            pub extern "C" fn drop(_self: *mut LssVecOfStringsResult) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Vecu8$clone"]
            pub extern "C" fn clone(_self: *mut Vec<u8>) -> *mut Vec<u8> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Vecu8$drop"]
            pub extern "C" fn drop(_self: *mut Vec<u8>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalString$clone"]
            pub extern "C" fn clone(_self: *mut OptionalString) -> *mut OptionalString {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalString$drop"]
            pub extern "C" fn drop(_self: *mut OptionalString) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainer$clone"]
            pub extern "C" fn clone(_self: *mut Vec<Container>) -> *mut Vec<Container> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecContainer$drop"]
            pub extern "C" fn drop(_self: *mut Vec<Container>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserApi$clone"]
            pub extern "C" fn clone(_self: *mut UserApi) -> *mut UserApi {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$UserApi$drop"]
            pub extern "C" fn drop(_self: *mut UserApi) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$GetStoragesExc$clone"]
            pub extern "C" fn clone(_self: *mut GetStoragesExc) -> *mut GetStoragesExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$GetStoragesExc$drop"]
            pub extern "C" fn drop(_self: *mut GetStoragesExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StorageTemplate$clone"]
            pub extern "C" fn clone(_self: *mut StorageTemplate) -> *mut StorageTemplate {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StorageTemplate$drop"]
            pub extern "C" fn drop(_self: *mut StorageTemplate) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$DeleteStorageExc$clone"]
            pub extern "C" fn clone(_self: *mut DeleteStorageExc) -> *mut DeleteStorageExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$DeleteStorageExc$drop"]
            pub extern "C" fn drop(_self: *mut DeleteStorageExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithStringExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<MnemonicPayload, StringExc>,
            ) -> *mut Result<MnemonicPayload, StringExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicPayloadResultWithStringExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<MnemonicPayload, StringExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$LssUsizeResult$clone"]
            pub extern "C" fn clone(_self: *mut LssUsizeResult) -> *mut LssUsizeResult {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$LssUsizeResult$drop"]
            pub extern "C" fn drop(_self: *mut LssUsizeResult) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ConfigParseExc$clone"]
            pub extern "C" fn clone(_self: *mut ConfigParseExc) -> *mut ConfigParseExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ConfigParseExc$drop"]
            pub extern "C" fn drop(_self: *mut ConfigParseExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$clone"]
            pub extern "C" fn clone(_self: *mut Container) -> *mut Container {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Container$drop"]
            pub extern "C" fn drop(_self: *mut Container) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Storage$clone"]
            pub extern "C" fn clone(_self: *mut Storage) -> *mut Storage {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$Storage$drop"]
            pub extern "C" fn drop(_self: *mut Storage) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithAddStorageExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<VoidType, AddStorageExc>,
            ) -> *mut Result<VoidType, AddStorageExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithAddStorageExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<VoidType, AddStorageExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicCreationExc$clone"]
            pub extern "C" fn clone(_self: *mut MnemonicCreationExc) -> *mut MnemonicCreationExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$MnemonicCreationExc$drop"]
            pub extern "C" fn drop(_self: *mut MnemonicCreationExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$LssBoolResult$clone"]
            pub extern "C" fn clone(_self: *mut LssBoolResult) -> *mut LssBoolResult {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$LssBoolResult$drop"]
            pub extern "C" fn drop(_self: *mut LssBoolResult) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalRustString$clone"]
            pub extern "C" fn clone(_self: *mut Option<String>) -> *mut Option<String> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalRustString$drop"]
            pub extern "C" fn drop(_self: *mut Option<String>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FoundationStorageApi$clone"]
            pub extern "C" fn clone(_self: *mut FoundationStorageApi) -> *mut FoundationStorageApi {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$FoundationStorageApi$drop"]
            pub extern "C" fn drop(_self: *mut FoundationStorageApi) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerUnmountExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<VoidType, ContainerUnmountExc>,
            ) -> *mut Result<VoidType, ContainerUnmountExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithContainerUnmountExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<VoidType, ContainerUnmountExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithForestMountExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<VoidType, ForestMountExc>,
            ) -> *mut Result<VoidType, ForestMountExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithForestMountExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<VoidType, ForestMountExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerMountExc$clone"]
            pub extern "C" fn clone(_self: *mut ContainerMountExc) -> *mut ContainerMountExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerMountExc$drop"]
            pub extern "C" fn drop(_self: *mut ContainerMountExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerUnmountExc$clone"]
            pub extern "C" fn clone(_self: *mut ContainerUnmountExc) -> *mut ContainerUnmountExc {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerUnmountExc$drop"]
            pub extern "C" fn drop(_self: *mut ContainerUnmountExc) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StorageTemplateResultWithFsaExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<StorageTemplate, FsaExc>,
            ) -> *mut Result<StorageTemplate, FsaExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$StorageTemplateResultWithFsaExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<StorageTemplate, FsaExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerResultWithCatlibExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<Container, CatlibExc>,
            ) -> *mut Result<Container, CatlibExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$ContainerResultWithCatlibExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<Container, CatlibExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithDeleteStorageExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<VoidType, DeleteStorageExc>,
            ) -> *mut Result<VoidType, DeleteStorageExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VoidTypeResultWithDeleteStorageExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<VoidType, DeleteStorageExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecRustString$clone"]
            pub extern "C" fn clone(_self: *mut Vec<String>) -> *mut Vec<String> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecRustString$drop"]
            pub extern "C" fn drop(_self: *mut Vec<String>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserRetrievalExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<CargoUser, UserRetrievalExc>,
            ) -> *mut Result<CargoUser, UserRetrievalExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserRetrievalExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<CargoUser, UserRetrievalExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalBytes$clone"]
            pub extern "C" fn clone(_self: *mut OptionalBytes) -> *mut OptionalBytes {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalBytes$drop"]
            pub extern "C" fn drop(_self: *mut OptionalBytes) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoConfig$clone"]
            pub extern "C" fn clone(_self: *mut CargoConfig) -> *mut CargoConfig {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoConfig$drop"]
            pub extern "C" fn drop(_self: *mut CargoConfig) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalContainer$clone"]
            pub extern "C" fn clone(_self: *mut Option<Container>) -> *mut Option<Container> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$OptionalContainer$drop"]
            pub extern "C" fn drop(_self: *mut Option<Container>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorageResultWithGetStoragesExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<Vec<Storage>, GetStoragesExc>,
            ) -> *mut Result<Vec<Storage>, GetStoragesExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$VecStorageResultWithGetStoragesExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<Vec<Storage>, GetStoragesExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserCreationExc$clone"]
            pub extern "C" fn clone(
                _self: *mut Result<CargoUser, UserCreationExc>,
            ) -> *mut Result<CargoUser, UserCreationExc> {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$CargoUserResultWithUserCreationExc$drop"]
            pub extern "C" fn drop(_self: *mut Result<CargoUser, UserCreationExc>) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$RustString$new"]
            pub extern "C" fn new() -> *mut String {
                Box::into_raw(Box::new(unsafe { String::new() }))
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$RustString$from_c_str"]
            pub extern "C" fn from_c_str(_self: *const std::os::raw::c_char) -> *mut String {
                unsafe {
                    Box::into_raw(Box::new(
                        std::ffi::CStr::from_ptr(_self).to_str().unwrap().to_owned(),
                    ))
                }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$RustString$as_mut_ptr"]
            pub extern "C" fn as_mut_ptr(_self: *mut String) -> *mut u8 {
                unsafe { (&mut *_self).as_mut_ptr() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$RustString$len"]
            pub extern "C" fn len(_self: *mut String) -> usize {
                unsafe { (&mut *_self).len() }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$RustString$clone"]
            pub extern "C" fn clone(_self: *mut String) -> *mut String {
                unsafe { Box::into_raw(Box::new((*_self).clone())) }
            }
        };
        const _: () = {
            #[doc(hidden)]
            #[export_name = "__rustybind__$RustString$drop"]
            pub extern "C" fn drop(_self: *mut String) -> () {
                unsafe {
                    #[allow(unused_must_use)]
                    {
                        Box::from_raw(_self);
                    }
                }
            }
        };
    }
}
