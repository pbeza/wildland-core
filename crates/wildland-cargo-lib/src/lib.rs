mod api;
mod cargo_lib;
mod errors;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;

pub use api::user::{MnemonicPayload, UserApi, UserPayload};
pub use cargo_lib::CargoLib;
use wildland_corex::{LocalSecureStorage, LssService, UserService};

/// Function creating [`CargoLib`] structure which is the main part of Cargo public API.
/// All functionalities are exposed to application side through this structure.
///
/// **Arguments**:
/// - lss: object implementing [`LocalSecureStorage`] trait
pub fn create_cargo_lib(lss: &'static dyn LocalSecureStorage) -> CargoLib {
    // TODO WILX-219 Memory leak
    _ = logging::init_subscriber();
    CargoLib::new(UserApi::new(UserService::new(LssService::new(lss))))
}
