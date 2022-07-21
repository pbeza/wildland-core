use crate::error::AdminManagerError;

mod api;

mod admin_manager;
mod error;
#[cfg(feature = "bindings")]
pub mod ffi;

pub use admin_manager::AdminManager;
pub use api::user::UserApi;

pub use wildland_corex::{SeedPhrase, SeedPhraseWordsArray};

pub type AdminManagerResult<T> = Result<T, AdminManagerError>;

pub fn create_admin_manager() -> AdminManager {
    AdminManager::default()
}
