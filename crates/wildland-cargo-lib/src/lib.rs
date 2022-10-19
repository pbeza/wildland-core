//! This crate provides a high level interface for the Cargo clients. It is built on top of the
//! Wildland CoreX library and provides Cargo specific abstractions like "user", "device" or
//! "sharing logic".
//!
//! All types and functions that are supposed to be exported from Rust library to other languages
//! are included within [`ffi`] module which is used by the **RustyBind** crate for generating
//! glue code and bindings to other languages.
//!
//! All Cargo functionalities can be accessed via [`api::CargoLib`] object. It aggregates and gives
//! access to API objects responsible for narrowed domains like [`api::UserApi`].
//!
//! [`api::CargoLib`] must be initialized with some set of parameters (see [`api::config`]).

pub mod api;
mod errors;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;
mod user;
