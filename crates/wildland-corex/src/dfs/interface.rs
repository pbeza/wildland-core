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

/// Getter exposed through ffi
impl Stat {
    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn access_time(&self) -> Option<UnixTimestamp> {
        self.access_time.clone()
    }
    pub fn modification_time(&self) -> Option<UnixTimestamp> {
        self.modification_time.clone()
    }
    pub fn change_time(&self) -> Option<UnixTimestamp> {
        self.change_time.clone()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub enum NodeType {
    File,
    Dir,
    Symlink,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UnixTimestamp {
    /// Number of seconds that elapsed since the beginning of the UNIX epoch
    pub sec: u64,
    /// fraction of a second expressed in nanoseconds
    pub nano_sec: u32,
}

/// Getter exposed through ffi
impl UnixTimestamp {
    pub fn sec(&self) -> u64 {
        self.sec
    }
    pub fn nano_sec(&self) -> u32 {
        self.nano_sec
    }
}

#[derive(Debug)]
pub struct FileDescriptor {}

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
    // Error probably will be eventually shown to a user as a text
    fn readdir(&mut self, path: String) -> Result<Vec<String>, DfsFrontendError>;
    fn getattr(&mut self, path: String) -> Result<Stat, DfsFrontendError>;

    /// Opens a file.
    ///
    /// Opening a file means initiating its state in DFS memory.
    ///
    /// Returns an error in case of file absence.
    fn open(&mut self, path: String) -> Result<FileDescriptor, DfsFrontendError>;
}
