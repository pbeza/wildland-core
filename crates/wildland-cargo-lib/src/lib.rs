mod api;
mod cargo_lib;
mod errors;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;

use std::sync::atomic::{AtomicBool, Ordering};

pub use api::*;
pub use cargo_lib::CargoLib;
use errors::SingleErrVariantResult;
use thiserror::Error;
use wildland_corex::{LocalSecureStorage, LssService, UserService};

use crate::errors::SingleVariantError;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("CargoLib creation error: {0}")]
pub struct CargoLibCreationError(pub String);

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Function creating [`CargoLib`] structure which is the main part of Cargo public API.
/// All functionalities are exposed to application side through this structure.
///
/// **Arguments**:
/// - lss: object implementing [`LocalSecureStorage`] trait
pub fn create_cargo_lib(
    lss: &'static dyn LocalSecureStorage,
    cfg: CargoConfig,
) -> SingleErrVariantResult<CargoLib, CargoLibCreationError> {
    // TODO WILX-219 Memory leak
    if !INITIALIZED.load(Ordering::Relaxed) {
        INITIALIZED.store(true, Ordering::Relaxed);
        logging::init_subscriber(cfg.logger_config)
            .map_err(|e| SingleVariantError::Failure(CargoLibCreationError(e)))?;
        Ok(CargoLib::new(
            UserApi::new(UserService::new(LssService::new(lss))),
            cfg.fsa_config,
        ))
    } else {
        Err(SingleVariantError::Failure(CargoLibCreationError(
            "CargoLib cannot be initialized twice.".to_string(),
        )))
    }
}
