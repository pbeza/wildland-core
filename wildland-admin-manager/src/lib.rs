use std::sync::Arc;

use admin_manager::AdminManager;
use api::{AdminManagerResult, SeedPhrase};

pub mod admin_manager;
pub mod api;

#[cfg(feature = "bindings")]
pub mod ffi;

pub fn create_seed_phrase() -> AdminManagerResult<SeedPhrase> {
    Ok(SeedPhrase::from([
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
        "test".to_owned(),
    ]))
}

pub fn create_admin_manager() -> AdminManager {
    admin_manager::AdminManager::default()
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
