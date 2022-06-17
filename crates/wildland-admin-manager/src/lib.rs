use admin_manager::AdminManager;
use api::{AdminManager as AdminManagerTrait, AdminManagerResult, SeedPhrase};

pub mod admin_manager;
pub mod api;

#[cfg(feature = "bindings")]
pub mod ffi;

pub fn create_seed_phrase() -> AdminManagerResult<SeedPhrase> {
    AdminManager::create_seed_phrase()
}

pub fn create_admin_manager() -> AdminManager {
    admin_manager::AdminManager::default()
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
