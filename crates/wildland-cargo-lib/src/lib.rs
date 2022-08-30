use crate::error::CargoLibError;
use std::rc::Rc;

mod api;

mod cargo_lib;
mod error;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;

pub use api::user::{MnemonicPayload, UserApi, UserPayload};
pub use cargo_lib::CargoLib;

use wildland_corex::{create_file_lss, LSSService, UserService};

pub type CargoLibResult<T> = Result<T, CargoLibError>;

// TODO:WILX-206 change lss_path to &dyn LocalSecureStorage and pass here native lss implementation after https://wildlandio.atlassian.net/browse/WILX-100 is finished
#[tracing::instrument]
pub fn create_cargo_lib(lss_path: String) -> CargoLibResult<CargoLib> {
    _ = logging::init_subscriber();
    let lss = Rc::new(create_file_lss(lss_path)?);
    let lss_service = Rc::new(LSSService::new(lss));
    let user_service = UserService::new(lss_service);
    let user_api = UserApi::new(user_service);
    Ok(CargoLib::new(user_api))
}
