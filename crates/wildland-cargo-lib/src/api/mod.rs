pub(crate) mod config;
pub(crate) mod user;

use thiserror::Error;
use wildland_corex::LocalSecureStorage;

use crate::{
    cargo_lib::CargoLib,
    errors::{CreationError, CreationResult},
    logging,
    user::UserService,
};

use self::{config::CargoCfg, user::UserApi};

#[cfg(test)]
use crate::test_utils::MockLssService;
#[cfg(not(test))]
use wildland_corex::LssService;

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
}
