mod api;
mod cargo_lib;
mod errors;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;
#[cfg(test)]
mod test_utils;
mod user;

pub use api::user::{MnemonicPayload, UserApi, UserPayload};
pub use cargo_lib::CargoLib;
#[cfg(test)]
use test_utils::MockLssService;
use user::UserService;
use wildland_corex::LocalSecureStorage;
#[cfg(not(test))]
use wildland_corex::LssService;

/// Function creating [`CargoLib`] structure which is the main part of Cargo public API.
/// All functionalities are exposed to application side through this structure.
///
/// **Arguments**:
/// - lss: object implementing [`LocalSecureStorage`] trait
#[cfg(not(test))]
pub fn create_cargo_lib(lss: &'static dyn LocalSecureStorage) -> CargoLib {
    // TODO WILX-219 Memory leak
    _ = logging::init_subscriber();
    CargoLib::new(UserApi::new(UserService::new(LssService::new(lss))))
}

#[cfg(test)]
pub fn create_cargo_lib(_lss: &'static dyn LocalSecureStorage) -> CargoLib {
    _ = logging::init_subscriber();
    CargoLib::new(UserApi::new(UserService::new(MockLssService::new())))
}
