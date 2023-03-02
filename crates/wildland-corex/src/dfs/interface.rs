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

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::PathResolutionError;

/// Represents filesystem statistics
#[derive(Debug, Clone)]
pub struct FsStat {
    /// file system block size
    pub block_size: u64,
    /// optimal transfer block size
    pub io_size: Option<u64>,
    /// Total data blocks in filesystem
    pub blocks: u64,
    /// Free blocks in filesystem
    pub free_blocks: u64,
    /// free blocks avail to non-superuser
    pub blocks_available: u64,
    /// Total number of nodes in filesystem
    pub nodes: u64,
    /// maximum filename length
    pub name_length: u64,
}

impl FsStat {
    pub fn block_size(&self) -> u64 {
        self.block_size
    }
    pub fn io_size(&self) -> Option<u64> {
        self.io_size
    }
    pub fn blocks(&self) -> u64 {
        self.blocks
    }
    pub fn free_blocks(&self) -> u64 {
        self.free_blocks
    }
    pub fn blocks_available(&self) -> u64 {
        self.blocks_available
    }
    pub fn nodes(&self) -> u64 {
        self.nodes
    }
    pub fn name_length(&self) -> u64 {
        self.name_length
    }
}

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
    pub permissions: WlPermissions,
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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

/// Representation of permissions supported on all target platforms
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WlPermissions {
    readonly: bool,
}

impl WlPermissions {
    pub fn readonly() -> Self {
        Self { readonly: true }
    }

    pub fn read_write() -> Self {
        Self { readonly: false }
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly
    }

    pub fn is_readonly(&self) -> bool {
        self.readonly
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum Operation {
    ReadDir,
    Metadata,
    Open,
    Close,
    CreateDir,
    RemoveDir,
    Read,
    Write,
    Seek,
    RemoveFile,
    CreateFile,
    Rename,
    SetPermission,
    SetOwner,
    SetLength,
    Sync,
    SetTimes,
    FileMetadata,
    SyncAll,
    SetFilePermissions,
    FileStatFs,
    StatFs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum Cause {
    UnsupportedBackendType,
    UnresponsiveBackend,
    AllBackendsUnresponsive,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub cause: Cause,
    pub operation: Option<Operation>,
    pub operation_path: Option<String>,
    pub backend_type: Option<String>,
}

impl Event {
    pub fn get_cause(&self) -> Cause {
        self.cause.clone()
    }

    pub fn get_operation(&self) -> Option<Operation> {
        self.operation.clone()
    }

    pub fn get_operation_path(&self) -> Option<String> {
        self.operation_path.clone()
    }

    pub fn get_backend_type(&self) -> Option<String> {
        self.backend_type.clone()
    }
}

pub trait EventSubscriber {
    fn pool_event(&self, millis: u64) -> Option<Event>;
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
    #[error("Parent of the provided path does not exist or is a file")]
    InvalidParent,
    #[error("Storages didn't respond")]
    StorageNotResponsive,
    #[error("Operation could not modify read-only path")]
    ReadOnlyPath,
    #[error("Directory is not empty")]
    DirNotEmpty,
    #[error("DFS Error: {0}")]
    Generic(String),
    #[error("Source path and target path are in different Containers")]
    MoveBetweenContainers,
    #[error("The new pathname contained a path prefix of the old, or, more generally, an attempt was made to make a directory a subdirectory of itself.")]
    SourceIsParentOfTarget,
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

    /// Opens a file. If file does not exist, it will not create one.
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

    /// Opens a file for update (reading and writing), first truncating the file to zero length
    /// if it exists or creating the file if it does not exist.
    ///
    /// # Errors:
    /// `InvalidParent` - parent directory does not exist
    fn create_file(&mut self, path: String) -> Result<FileHandle, DfsFrontendError>;

    /// Removes a file
    ///
    /// # Errors:
    /// - `NoSuchPath` - requested path does not exist
    /// - `NotAFile` - provided path represents a node that is not a file
    fn remove_file(&mut self, path: String) -> Result<(), DfsFrontendError>;

    /// Rename a file or directory to a new path, if new path does not exist yet.
    /// In contrast to POSIX-like rename operation, it returns error in case of new path existence
    /// in all cases, so it is up to a caller whether to remove a node under new path or not.
    ///
    /// # Errors:
    /// `NoSuchPath` - source not found
    /// `SourceIsParentOfTarget` - new directory would be a subdirectory of itself
    /// `MoveBetweenContainers` - `new_path` is in different Container than `old_path`
    /// `PathAlreadyExists` - `new_path` already exists
    fn rename(&mut self, old_path: String, new_path: String) -> Result<(), DfsFrontendError>;

    /// Changes the permissions of the underlying file.
    fn set_permissions(
        &mut self,
        path: String,
        permissions: WlPermissions,
    ) -> Result<(), DfsFrontendError>;

    /// Changes the permissions of the file.
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn set_file_permissions(
        &mut self,
        file: &FileHandle,
        permissions: WlPermissions,
    ) -> Result<(), DfsFrontendError>;

    /// Not supported yet - it always returns `DfsFrontendError::Generic(_)`
    fn set_owner(&mut self, path: String) -> Result<(), DfsFrontendError>;

    /// Creates a new, empty directory at the provided path
    ///
    /// # Errors:
    /// `InvalidParent` - a parent of the given path doesn’t exist.
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
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn read(&mut self, file: &FileHandle, count: usize) -> Result<Vec<u8>, DfsFrontendError>;

    /// Tries to write bytes from the buffer to a file and returns amount of actually written bytes
    /// which can be different from buf length.
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn write(&mut self, file: &FileHandle, buf: Vec<u8>) -> Result<usize, DfsFrontendError>;

    /// Truncates or extends the underlying file, updating the size of this file to become `length`.
    /// If the size is less than the current file’s size, then the file will be shrunk. If it is greater
    /// than the current file’s size, then the file will be extended to size and have all of the intermediate
    /// data filled in with 0s.
    /// If the file’s cursor was further than the new length then the file is
    /// shrunk using this operation, the cursor will now be at the new end of file.
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn set_length(&mut self, file: &FileHandle, length: usize) -> Result<(), DfsFrontendError>;

    /// Attempts to sync all data and metadata to storage.
    ///
    /// This function will attempt to ensure that all in-memory data reaches the storage before returning.
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn sync(&mut self, file: &FileHandle) -> Result<(), DfsFrontendError>;

    /// Changes the timestamps of the underlying file.
    ///
    /// Passing None as an argument means not overwriting given parameter (not setting it to None)
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn set_times(
        &mut self,
        file: &FileHandle,
        access_time: Option<UnixTimestamp>,
        modification_time: Option<UnixTimestamp>,
    ) -> Result<(), DfsFrontendError>;

    /// Queries metadata about the underlying file.
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn file_metadata(&mut self, file: &FileHandle) -> Result<Stat, DfsFrontendError>;

    /// Returns information about a mounted filesystem containing the `file`.
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn file_stat_fs(&mut self, file: &FileHandle) -> Result<FsStat, DfsFrontendError>;

    /// Seek to an offset, in bytes from the beginning of a file.
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn seek_from_start(
        &mut self,
        file: &FileHandle,
        pos_from_start: u64,
    ) -> Result<usize, DfsFrontendError>;

    /// Seek to an offset, in bytes from the current cursor position of a file.
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn seek_from_current(
        &mut self,
        file: &FileHandle,
        pos_from_current: i64,
    ) -> Result<usize, DfsFrontendError>;

    /// Seek to an offset, in bytes from the end of a file.
    /// Negative argument means moving a cursor back (std-like approach).
    ///
    /// # Errors:
    /// - `FileAlreadyClosed` - DFS does not have the file's state, meaning it's been already close.
    fn seek_from_end(
        &mut self,
        file: &FileHandle,
        pos_from_end: i64,
    ) -> Result<usize, DfsFrontendError>;

    /// Attempts to sync all metadata and data of all opened files in DFS Context.
    fn sync_all(&mut self) -> Result<(), DfsFrontendError>;

    /// Returns information about a mounted filesystem. Path is the pathname of any file within the
    /// mounted filesystem.
    ///
    /// # Errors:
    /// `NoSuchPath` - no such path exists
    fn stat_fs(&mut self, path: String) -> Result<FsStat, DfsFrontendError>;

    /// Returns subscriber that can listen to DFS events.
    /// Events may be split between different `EventSubscriber`.
    fn get_subscriber(&self) -> Arc<Mutex<dyn EventSubscriber>>;
}
