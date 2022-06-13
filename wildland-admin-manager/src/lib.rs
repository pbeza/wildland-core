use admin_manager::AdminManager;
use api::AdminManagerResult;
use wildland_corex::FileWallet;

pub mod admin_manager;
pub mod api;

#[cfg(feature = "bindings")]
pub mod ffi;

pub fn create_file_wallet_admin_manager() -> AdminManagerResult<AdminManager<FileWallet>> {
    admin_manager::AdminManager::<FileWallet>::new()
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
