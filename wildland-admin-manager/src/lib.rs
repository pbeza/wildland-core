#![feature(get_mut_unchecked)]

pub mod admin_manager;
pub mod api;
#[cfg(feature = "bindings")]
pub mod ffi;

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
