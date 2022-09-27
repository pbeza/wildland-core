pub(crate) mod config;
pub(crate) mod user;

use self::{
    config::{CargoCfgProvider, CargoConfig},
    user::UserApi,
};
use crate::{
    cargo_lib::CargoLib,
    errors::{SingleErrVariantResult, SingleVariantError},
    logging,
    user::UserService,
};
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;
use wildland_corex::LocalSecureStorage;

#[cfg(test)]
use crate::test_utils::MockLssService;
#[cfg(not(test))]
use wildland_corex::LssService;

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
        logging::init_subscriber(cfg.get_log_level(), cfg.get_log_file())
            .map_err(|e| SingleVariantError::Failure(CargoLibCreationError(e)))?;

        #[cfg(not(test))]
        return Ok(CargoLib::new(UserApi::new(UserService::new(
            LssService::new(lss),
        ))));

        #[cfg(test)]
        {
            let _ = lss;
            Ok(CargoLib::new(UserApi::new(UserService::new(
                MockLssService::new(),
            ))))
        }
    } else {
        Err(SingleVariantError::Failure(CargoLibCreationError(
            "CargoLib cannot be initialized twice.".to_string(),
        )))
    }
}
