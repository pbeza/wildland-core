pub mod api;
pub mod admin_manager;

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
