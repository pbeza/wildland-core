mod api;
mod cargo_lib;
mod errors;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;

use api::config::{CargoCfgProvider, CargoConfig};
pub use api::user::{MnemonicPayload, UserApi, UserPayload};
pub use cargo_lib::CargoLib;
use errors::{CreationError, CreationResult};
use log::info;
use thiserror::Error;
use wildland_corex::{LocalSecureStorage, LssService, UserService};

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
    config_provider: &'static dyn CargoCfgProvider,
) -> CreationResult<CargoLib, CargoLibCreationError> {
    // TODO WILX-219 Memory leak
    let cfg: CargoConfig = serde_json::from_slice(&config_provider.get_config())
        .map_err(|e| CreationError::NotCreated(CargoLibCreationError(e.to_string())))?;
    _ = logging::init_subscriber(cfg.logger.clone());
    info!("CargoLib initialized with config: {cfg:?}");
    Ok(CargoLib::new(UserApi::new(UserService::new(
        LssService::new(lss),
    ))))
}
