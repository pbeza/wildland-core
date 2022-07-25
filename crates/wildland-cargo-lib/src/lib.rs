use crate::error::CargoLibError;

mod api;

mod cargo_lib;
mod error;
#[cfg(feature = "bindings")]
pub mod ffi;

pub use api::user::UserApi;
pub use cargo_lib::CargoLib;

pub use wildland_corex::{SeedPhrase, SeedPhraseWordsArray};

pub type CargoLibResult<T> = Result<T, CargoLibError>;

pub fn create_cargo_lib() -> CargoLib {
    CargoLib::default()
}
