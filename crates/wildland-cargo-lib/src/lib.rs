mod api;
mod cargo_lib;
mod error;
#[cfg(feature = "bindings")]
pub mod ffi;

use crate::error::CargoLibError;
pub use api::user::{MnemonicPayload, UserApi};
pub use cargo_lib::CargoLib;
use std::rc::Rc;
use wildland_corex::create_file_lss;

pub type CargoLibResult<T> = Result<T, CargoLibError>;

pub fn create_cargo_lib(lss_path: String) -> CargoLibResult<CargoLib> {
    let lss = create_file_lss(lss_path)?;
    Ok(CargoLib::new(Rc::new(lss)))
}
