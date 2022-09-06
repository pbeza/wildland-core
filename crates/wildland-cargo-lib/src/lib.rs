mod api;
mod cargo_lib;
mod error;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;

pub use api::user::{MnemonicPayload, UserApi, UserPayload};
pub use cargo_lib::CargoLib;
use error::{CreationError, CreationResult};
use std::rc::Rc;
use wildland_corex::{create_file_lss, LSSService, LssError, UserService};

// TODO change lss_path to &dyn LocalSecureStorage and pass here native lss implementation after https://wildlandio.atlassian.net/browse/WILX-100 is finished
#[tracing::instrument]
pub fn create_cargo_lib(lss_path: String) -> CreationResult<CargoLib, LssError> {
    _ = logging::init_subscriber(); // TODO WILX-219 Memory leak
    let file_lss = create_file_lss(lss_path).map_err(CreationError::NotCreated)?;
    let lss = Rc::new(file_lss);
    let lss_service = Rc::new(LSSService::new(lss));
    let user_service = UserService::new(lss_service);
    let user_api = UserApi::new(user_service);
    Ok(CargoLib::new(user_api))
}
