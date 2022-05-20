pub mod admin_manager;
pub mod ffi;

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
