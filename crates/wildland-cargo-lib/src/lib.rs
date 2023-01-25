//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
pub mod errors;
#[cfg(feature = "bindings")]
pub mod ffi;
mod logging;
mod templates;
#[cfg(test)]
mod tests;
mod user;
mod utils;
