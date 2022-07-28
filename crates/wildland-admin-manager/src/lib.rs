use crate::error::AdminManagerError;
use std::rc::Rc;

mod api;

mod admin_manager;
mod error;
#[cfg(feature = "bindings")]
pub mod ffi;

pub use admin_manager::AdminManager;
pub use api::user::MnemonicPayload;
pub use api::user::UserApi;

use wildland_corex::{create_file_lss, LSSService, UserService};

pub type AdminManagerResult<T> = Result<T, AdminManagerError>;

// TODO change lss_path to &dyn LocalSecureStorage and pass here native lss implementation after https://wildlandio.atlassian.net/browse/WILX-100 is finished
pub fn create_admin_manager(lss_path: String) -> AdminManagerResult<AdminManager> {
    let lss = Rc::new(create_file_lss(lss_path)?);
    let lss_service = Rc::new(LSSService::new(lss));
    let user_service = UserService::new(lss_service);
    let user_api = UserApi::new(user_service);
    Ok(AdminManager::new(user_api))
}
