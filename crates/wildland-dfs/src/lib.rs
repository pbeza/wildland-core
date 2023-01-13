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

#![cfg_attr(test, feature(io_error_more))]

pub mod encrypted;
pub mod storage_backend;
pub mod unencrypted;

pub use wildland_corex::dfs::interface::*;
pub use wildland_corex::{
    Storage,
    StorageTemplate,
    StorageTemplateError,
    CONTAINER_NAME_PARAM,
    CONTAINER_UUID_PARAM,
};
