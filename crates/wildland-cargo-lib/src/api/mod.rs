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
use std::{
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use thiserror::Error;
use wildland_corex::LocalSecureStorage;

#[cfg(test)]
use crate::test_utils::MockLssService as LssService;
#[cfg(not(test))]
use wildland_corex::LssService;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("CargoLib creation error: {0}")]
pub struct CargoLibCreationError(pub String);

static INITIALIZED: AtomicBool = AtomicBool::new(false);

type SharedCargoLib = Arc<Mutex<CargoLib>>;
static mut CARGO_LIB: MaybeUninit<SharedCargoLib> = MaybeUninit::uninit();

/// Function creating [`CargoLib`] structure which is the main part of Cargo public API.
/// All functionalities are exposed to application side through this structure.
///
pub fn create_cargo_lib(
    lss: &'static dyn LocalSecureStorage,
    cfg: CargoConfig,
) -> SingleErrVariantResult<SharedCargoLib, CargoLibCreationError> {
    // TODO WILX-219 Memory leak

    if !INITIALIZED.load(Ordering::Relaxed) {
        INITIALIZED.store(true, Ordering::Relaxed);

        logging::init_subscriber(cfg.get_log_level(), cfg.get_log_file())
            .map_err(|e| SingleVariantError::Failure(CargoLibCreationError(e)))?;

        let cargo_lib = Arc::new(Mutex::new(CargoLib::new(UserApi::new(UserService::new(
            LssService::new(lss),
        )))));

        unsafe {
            CARGO_LIB.write(cargo_lib);
        }
    }
    unsafe { Ok(CARGO_LIB.assume_init_ref().clone()) }
}
