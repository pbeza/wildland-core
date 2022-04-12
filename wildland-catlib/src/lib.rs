pub mod bridge;
pub mod container;
pub mod forest;
pub mod storage;

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
