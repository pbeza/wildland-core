pub mod admin_manager;
pub mod api;

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
