use admin_manager::AdminManager;
use wildland_corex::file_wallet_factory;

pub mod admin_manager;
pub mod api;

#[cfg(feature = "bindings")]
pub mod ffi;

pub fn create_admin_manager() -> AdminManager {
    admin_manager::AdminManager::with_wallet_factory(&file_wallet_factory)
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
