mod api;
mod cargo_lib;
mod errors;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;

pub use api::user::{MnemonicPayload, UserApi, UserPayload};
pub use cargo_lib::CargoLib;
use wildland_corex::{LocalSecureStorage, LssService, UserService};

pub fn create_cargo_lib(lss: &'static dyn LocalSecureStorage) -> CargoLib {
    // TODO WILX-219 Memory leak
    _ = logging::init_subscriber();
    CargoLib::new(UserApi::new(UserService::new(LssService::new(lss))))
}
