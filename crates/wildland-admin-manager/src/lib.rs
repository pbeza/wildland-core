use crate::api::AdminManagerError;
use admin_manager::AdminManager;
use wildland_corex::create_file_wallet;

pub mod admin_manager;
pub mod api;

mod error;
#[cfg(feature = "bindings")]
pub mod ffi;

pub type AdminManagerResult<T> = Result<T, AdminManagerError>;

pub fn create_admin_manager() -> AdminManager {
    AdminManager::with_wallet(create_file_wallet().unwrap())
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
