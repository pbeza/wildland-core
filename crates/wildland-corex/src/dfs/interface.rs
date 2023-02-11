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
/// Represents metadata of a filesystem node
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
///
/// DFS methods may return error that are not related with particular operation but rather
/// with wildland system in general. Those errors could be:
///
/// - `PathResolutionError` - happens when path resolving failed, e.g. due to the catlib error.
/// - `StorageNotResponsive` - happens when none of storages that operation involves returns an answer.
/// - `Generic` - unanticipated errors.
pub trait DfsFrontend {
    /// Returns vector of entries found under the provided path.
    /// It may merge entries from multiple containers.
    ///
    /// # Errors:
    /// - `NotADirectory` - for paths that don't represent directories
    /// - `NoSuchPath` - requested path does not exist
    ///
    fn read_dir(&mut self, path: String) -> Result<Vec<String>, DfsFrontendError>;

    /// Returns metadata of a node.
    ///
    /// # Errors:
    ///
    /// - `NoSuchPath` - requested path does not exist
    fn metadata(&mut self, path: String) -> Result<Stat, DfsFrontendError>;

    /// Opens a file.
    ///
    /// Opening a file means initiating its state in DFS memory.
    ///
    /// # Errors:
    /// - `NoSuchPath` - requested path does not exist
    /// - `NotAFile` - provided path represents a node that is not a file
    fn open(&mut self, path: String) -> Result<FileHandle, DfsFrontendError>;

    /// Closes a file (removes file's state from DFS) referred by the provided `FileHandle`.
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn close(&mut self, file: &FileHandle) -> Result<(), DfsFrontendError>;

    /// Removes a file
    ///
    /// # Errors:
    /// - `NoSuchPath` - requested path does not exist
    /// - `NotAFile` - provided path represents a node that is not a file
    fn remove_file(&mut self, path: String) -> Result<(), DfsFrontendError>;

    /// Creates a new, empty directory at the provided path
    ///
    /// # Errors:
    /// `ParentDoesNotExist` - a parent of the given path doesn’t exist.
    /// `PathAlreadyExists` - path already exists.
    fn create_dir(&mut self, path: String) -> Result<(), DfsFrontendError>;

    /// Removes an empty directory
    ///
    /// # Errors:
    /// `NotADirectory` - path does not represent a directory
    /// `NoSuchPath` - no such path exists
    /// `DirNotEmpty` - directory is not empty
    fn remove_dir(&mut self, path: String) -> Result<(), DfsFrontendError>;

    /// Reads number of bytes (specified by the `count` arg) returned as a vector.
    /// Vector length represents number of bytes that were actually read, it may be less than
    /// the requested number of bytes.
    fn read(&mut self, file: &FileHandle, count: usize) -> Result<Vec<u8>, DfsFrontendError>;

    /// Tries to write bytes from the buffer to a file and returns amount of actually written bytes
    /// which can be different from buf length.
    fn write(&mut self, file: &FileHandle, buf: Vec<u8>) -> Result<usize, DfsFrontendError>;

    /// Seek to an offset, in bytes from the beginning of a file.
    fn seek_from_start(
        &mut self,
        file: &FileHandle,
        pos_from_start: u64,
    ) -> Result<usize, DfsFrontendError>;
    /// Seek to an offset, in bytes from the current cursor position of a file.
    fn seek_from_current(
        &mut self,
        file: &FileHandle,
        pos_from_current: i64,
    ) -> Result<usize, DfsFrontendError>;
    /// Seek to an offset, in bytes from the end of a file.
    /// Negative argument means moving a cursor back (std-like approach).
    fn seek_from_end(
        &mut self,
        file: &FileHandle,
        pos_from_end: i64,
    ) -> Result<usize, DfsFrontendError>;
}
