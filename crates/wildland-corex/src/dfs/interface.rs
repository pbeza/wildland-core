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

use thiserror::Error;

use crate::PathResolutionError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stat {
    pub node_type: NodeType,
    /// size in bytes
    pub size: u64,
    /// Some nodes, like virtual ones, may have time properties set to None
    pub access_time: Option<UnixTimestamp>,
    pub modification_time: Option<UnixTimestamp>,
    pub change_time: Option<UnixTimestamp>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodeType {
    File,
    Dir,
    Symlink,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnixTimestamp {
    pub sec: u64,
    pub nano_sec: u32,
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum DfsFrontendError {
    #[error("Path does not exist")]
    NoSuchPath,
    #[error(transparent)]
    PathResolutionError(#[from] PathResolutionError),
    #[error("DFS Error: {0}")]
    Generic(String),
}

/// Interface that DFS should expose towards frontends
pub trait DfsFrontend {
    // Error probably will be eventually shown to a user
    fn readdir(&mut self, path: String) -> Result<Vec<String>, DfsFrontendError>;
    fn getattr(&mut self, path: String) -> Result<Stat, DfsFrontendError>;
}
