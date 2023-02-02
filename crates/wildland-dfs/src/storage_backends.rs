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

pub mod s3;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use wildland_corex::dfs::interface::{DfsFrontendError, Stat};
use wildland_corex::Storage;

use crate::close_on_drop_descriptor::CloseOnDropDescriptor;

#[derive(thiserror::Error, Debug)]
pub enum StorageBackendError {
    #[error(transparent)]
    Generic(#[from] anyhow::Error),
}

impl From<std::io::Error> for StorageBackendError {
    fn from(e: std::io::Error) -> Self {
        Self::Generic(e.into())
    }
}
impl From<std::path::StripPrefixError> for StorageBackendError {
    fn from(e: std::path::StripPrefixError) -> Self {
        Self::Generic(e.into())
    }
}

pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

#[derive(thiserror::Error, Debug)]
pub enum CloseError {
    #[error("File has been already closed")]
    FileAlreadyClosed,
}

/// FileDescriptor contains state of opened file and definition of how it is stored, therefore
/// it is backend specific, cause file can be stored in different ways (e.g. partitioned depending
/// on the backend's type) and e.g. seek operation may be implemented differently.
pub trait OpenedFileDescriptor: std::fmt::Debug {
    fn close(&self) -> Result<(), CloseError>;
    /// Reads number of bytes specified by the `count` parameter and advances inner cursor of the
    /// opened file.
    ///
    /// Returns vector of bytes which can have length smaller than requested.
    fn read(&mut self, count: usize) -> Result<Vec<u8>, DfsFrontendError>;

    /// Writes bytes at the current cursor position and returns number of written bytes.
    fn write(&mut self, buf: &[u8]) -> Result<usize, DfsFrontendError>;

    /// Changes inner cursor position.
    fn seek(&mut self, seek_from: SeekFrom) -> Result<u64, DfsFrontendError>;
}

#[derive(Debug)]
pub enum OpenResponse {
    Found(CloseOnDropDescriptor),
    NotAFile,
    NotFound,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReaddirResponse {
    Entries(Vec<PathBuf>),
    NotADirectory,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GetattrResponse {
    Found(Stat),
    NotFound,
}

/// Error represents scenario when data could not be retrieved from the StorageBackend, e.g. some
/// network error. This mean that operation can be called again later of data can still be successfully
/// retrieved from another equivalent backend.
///
/// All logical errors, e.g. trying opening directory, should be reflected in the inner type, like OpenResponse.
/// Those variants are hidden inside Ok value because they should not trigger retrying operation.
pub trait StorageBackend {
    fn readdir(&self, path: &Path) -> Result<ReaddirResponse, StorageBackendError>;
    fn getattr(&self, path: &Path) -> Result<GetattrResponse, StorageBackendError>;
    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError>;
}

pub trait StorageBackendFactory {
    fn init_backend(&self, storage: Storage) -> anyhow::Result<Rc<dyn StorageBackend>>;
}
