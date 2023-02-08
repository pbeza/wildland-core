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
use uuid::Uuid;

use crate::PathResolutionError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stat {
    pub node_type: NodeType,
    /// size in bytes
    pub size: usize,
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

    pub fn size(&self) -> usize {
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
    Other,
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

#[derive(Debug, Clone)]
pub struct FileHandle {
    pub descriptor_uuid: Uuid,
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum DfsFrontendError {
    #[error("Operation not permitted on other nodes than files")]
    NotAFile,
    #[error("Operation not permitted on other nodes than directories")]
    NotADirectory,
    #[error("Path does not exist")]
    NoSuchPath,
    #[error("This file handle has been already closed")]
    FileAlreadyClosed,
    #[error("Seek cannot be performed")]
    SeekError,
    #[error("Concurrent issue detected")]
    ConcurrentIssue,
    #[error(transparent)]
    PathResolutionError(#[from] PathResolutionError),
    #[error("Path already exists")]
    PathAlreadyExists,
    #[error("Parent of the provided path does not exist")]
    ParentDoesNotExist,
    #[error("Storages didn't respond")]
    StorageNotResponsive,
    #[error("Operation could not modify read-only path")]
    ReadOnlyPath,
    #[error("Directory is not empty")]
    DirNotEmpty,
    #[error("DFS Error: {0}")]
    Generic(String),
}

impl From<std::io::Error> for DfsFrontendError {
    fn from(err: std::io::Error) -> Self {
        DfsFrontendError::Generic(err.to_string())
    }
}

/// Interface that DFS exposes towards filesystem-like frontend providers
pub trait DfsFrontend {
    /// Returns vector of entries found under the provided path.
    /// It may merge entries from multiple containers.
    ///
    /// # Errors:
    /// - `NotADirectory` - for paths that don't represent directories
    /// - `NoSuchPath` - requested path does not exist
    fn readdir(&mut self, path: String) -> Result<Vec<String>, DfsFrontendError>;

    fn getattr(&mut self, path: String) -> Result<Stat, DfsFrontendError>;

    /// Opens a file.
    ///
    /// Opening a file means initiating its state in DFS memory.
    ///
    /// Returns an error in case of file absence.
    fn open(&mut self, path: String) -> Result<FileHandle, DfsFrontendError>;
    fn close(&mut self, file: &FileHandle) -> Result<(), DfsFrontendError>;

    fn remove_file(&mut self, path: String) -> Result<(), DfsFrontendError>;

    fn create_dir(&mut self, path: String) -> Result<(), DfsFrontendError>;

    /// Succeeds if the directory exists and is empty
    fn remove_dir(&mut self, path: String) -> Result<(), DfsFrontendError>;

    fn read(&mut self, file: &FileHandle, count: usize) -> Result<Vec<u8>, DfsFrontendError>;
    fn write(&mut self, file: &FileHandle, buf: Vec<u8>) -> Result<usize, DfsFrontendError>;
    fn seek_from_start(
        &mut self,
        file: &FileHandle,
        pos_from_start: usize,
    ) -> Result<usize, DfsFrontendError>;
    fn seek_from_current(
        &mut self,
        file: &FileHandle,
        pos_from_current: isize,
    ) -> Result<usize, DfsFrontendError>;
    fn seek_from_end(
        &mut self,
        file: &FileHandle,
        pos_from_end: usize,
    ) -> Result<usize, DfsFrontendError>;
}
