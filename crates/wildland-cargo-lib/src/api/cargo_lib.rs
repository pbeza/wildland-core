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

use crate::{
    api::{
        config::{CargoConfig, FoundationStorageApiConfig},
        foundation_storage::FoundationStorageApi,
        user::UserApi,
    },
    logging,
    user::UserService,
};
use std::{
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use thiserror::Error;
use wildland_corex::{LocalSecureStorage, LssService};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[repr(C)]
pub enum CargoLibCreationError {
    #[error("CargoLib creation error: {0}")]
    Error(String),
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

type SharedCargoLib = Arc<Mutex<CargoLib>>;
static mut CARGO_LIB: MaybeUninit<SharedCargoLib> = MaybeUninit::uninit();

/// Structure aggregating and exposing public API of CargoLib library.
/// All functionalities are exposed to application side through this structure.
///
/// It can be created with [`create_cargo_lib`] function.
///
#[derive(Clone)]
pub struct CargoLib {
    user_api: UserApi,
    foundation_storage_api: FoundationStorageApi,
}

impl CargoLib {
    pub fn new(
        lss: &'static dyn LocalSecureStorage,
        fsa_config: FoundationStorageApiConfig,
    ) -> Self {
        let lss_service = LssService::new(lss);
        Self {
            user_api: UserApi::new(UserService::new(lss_service.clone())),
            foundation_storage_api: FoundationStorageApi::new(fsa_config, lss_service),
        }
    }

    /// Returns structure aggregating API for user management
    #[tracing::instrument(skip(self))]
    pub fn user_api(&self) -> UserApi {
        self.user_api.clone()
    }

    /// Returns structure aggregating API for Foundation Storage management
    #[tracing::instrument(skip(self))]
    pub fn foundation_storage_api(&self) -> FoundationStorageApi {
        self.foundation_storage_api.clone()
    }
}

/// [`CargoLib`] initializer which is the main part of Cargo public API.
/// All functionalities are exposed to application side through this structure.
///
/// Underlying structure is created only once, subsequent call will return handle to the same structure.
///
/// It requires the following arguments:
/// - lss: some type implementing [`LocalSecureStorage`] trait. It is usually provided by the native
/// to a target platform language. It is assumed that a lss reference should be valid for a whole
/// program execution (static lifetime).
/// - cfg: [`CargoConfig`] structure with config variables (logger, endpoints, etc.)
///
/// CargoLib expects to get references with static lifetimes so it is important not to inline
/// objects (e.g. LSS) initialization along with createCargoLib call.
///
/// ```
/// # use wildland_corex::{LocalSecureStorage, LssResult};
/// # use wildland_cargo_lib::api::{config::*, cargo_lib::create_cargo_lib};
/// # use tracing::Level;
/// #
/// struct TestLss{};
///
/// impl LocalSecureStorage for TestLss {
/// // ...implementation here
/// #    fn insert(&self, key: String, value: String) -> LssResult<Option<String>>{todo!()}
/// #    fn get(&self, key: String) -> LssResult<Option<String>>{todo!()}
/// #    fn contains_key(&self, key: String) -> LssResult<bool>{todo!()}
/// #    fn keys(&self) -> LssResult<Vec<String>>{todo!()}
/// #    fn keys_starting_with(&self, prefix: String) -> LssResult<Vec<String>>{todo!()}
/// #    fn remove(&self, key: String) -> LssResult<Option<String>>{todo!()}
/// #    fn len(&self) -> LssResult<usize>{todo!()}
/// #    fn is_empty(&self) -> LssResult<bool>{todo!()}
/// }
///
/// let lss = TestLss{};
///
/// # use std::path::PathBuf;
/// let cfg = CargoConfig{     
///     logger_config: LoggerConfig {
///         use_logger: true,
///         log_level: Level::TRACE,
///         log_use_ansi: false,
///         log_file_path: PathBuf::from("cargo_lib_log"),
///         log_file_rotate_directory: PathBuf::from(".".to_owned()),
///         log_file_enabled: true,
///         oslog_category: None,
///         oslog_subsystem: None,
///     },
///     fsa_config: FoundationStorageApiConfig {
///         evs_url: "some_url".to_owned(),
///         sc_url: "some_url".to_owned(),
///     },
/// };
///
/// let lss: &'static TestLss = unsafe { std::mem::transmute(&lss) };
/// let cargo_lib = create_cargo_lib(lss, cfg);
/// ```
pub fn create_cargo_lib(
    lss: &'static dyn LocalSecureStorage,
    cfg: CargoConfig,
) -> Result<SharedCargoLib, CargoLibCreationError> {
    if !INITIALIZED.load(Ordering::Relaxed) {
        INITIALIZED.store(true, Ordering::Relaxed);

        logging::init_subscriber(cfg.logger_config)
            .map_err(|e| CargoLibCreationError::Error(e.to_string()))?;

        let cargo_lib = Arc::new(Mutex::new(CargoLib::new(lss, cfg.fsa_config)));

        unsafe {
            CARGO_LIB.write(cargo_lib);
        }
    }
    unsafe { Ok(CARGO_LIB.assume_init_ref().clone()) }
}
