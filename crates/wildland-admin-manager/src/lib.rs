use crate::error::AdminManagerError;
use std::rc::Rc;

mod api;

mod admin_manager;
mod error;
#[cfg(feature = "bindings")]
pub mod ffi;

pub use admin_manager::AdminManager;
pub use api::user::UserApi;

use wildland_corex::FileLSS;
pub use wildland_corex::SeedPhrase;

pub type AdminManagerResult<T> = Result<T, AdminManagerError>;

pub fn create_file_lss(path: String) -> AdminManagerResult<FileLSS> {
    wildland_corex::create_file_lss(path).map_err(AdminManagerError::from)
}

// TODO change lss to &dyn LocalSecureStorage after https://wildlandio.atlassian.net/browse/WILX-100 is finished
pub fn create_admin_manager(lss: FileLSS) -> AdminManager {
    AdminManager::new(Rc::new(lss))
}
