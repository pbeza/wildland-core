mod api;
mod cargo_lib;
mod errors;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;

use api::config::CargoCfg;
pub use api::user::{MnemonicPayload, UserApi, UserPayload};
pub use cargo_lib::CargoLib;
use errors::CreationResult;
use thiserror::Error;
use wildland_corex::{LocalSecureStorage, LssService, UserService};

use crate::errors::CreationError;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("CargoLib creation error: {0}")]
pub struct CargoLibCreationError(pub String);

/// Function creating [`CargoLib`] structure which is the main part of Cargo public API.
/// All functionalities are exposed to application side through this structure.
///
/// **Arguments**:
/// - lss: object implementing [`LocalSecureStorage`] trait
pub fn create_cargo_lib(
    lss: &'static dyn LocalSecureStorage,
    cfg: &'static dyn CargoCfg,
) -> CreationResult<CargoLib, CargoLibCreationError> {
    // TODO WILX-219 Memory leak
    logging::init_subscriber(cfg.get_log_level(), cfg.get_log_file())
        .map_err(|e| CreationError::NotCreated(CargoLibCreationError(e)))?;
    Ok(CargoLib::new(UserApi::new(UserService::new(
        LssService::new(lss),
    ))))
}
