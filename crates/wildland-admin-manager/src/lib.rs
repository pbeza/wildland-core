use admin_manager::AdminManager;
use wildland_corex::create_file_wallet;
use crate::api::AdminManagerError;

pub mod admin_manager;
pub mod api;

#[cfg(feature = "bindings")]
pub mod ffi;
mod error;

pub type AdminManagerResult<T> = Result<T, AdminManagerError>;

pub fn create_admin_manager() -> AdminManager {
    AdminManager::with_wallet(create_file_wallet().unwrap())
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
