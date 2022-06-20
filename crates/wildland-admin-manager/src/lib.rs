use admin_manager::AdminManager;

pub mod admin_manager;
pub mod api;

#[cfg(feature = "bindings")]
pub mod ffi;

pub fn create_admin_manager() -> AdminManager {
    admin_manager::AdminManager::default()
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
