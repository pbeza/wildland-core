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

use std::path::{Path, PathBuf};
use std::rc::Rc;

use wildland_corex::dfs::interface::{OpenedFileDescriptor, Stat};

#[derive(thiserror::Error, Debug)]
pub enum StorageBackendError {
    #[error("Operation not permitted for paths that don't represent directories")]
    NotADirectory,
    #[error(transparent)]
    Generic(anyhow::Error),
}

impl From<std::io::Error> for StorageBackendError {
    fn from(e: std::io::Error) -> Self {
        StorageBackendError::Generic(e.into())
    }
}
impl From<std::path::StripPrefixError> for StorageBackendError {
    fn from(e: std::path::StripPrefixError) -> Self {
        StorageBackendError::Generic(e.into())
    }
}

#[derive(Debug)]
pub enum OpenResponse {
    Found(Rc<dyn OpenedFileDescriptor>),
    NotAFile,
    NotFound,
}

/// Error represents scenario when data could not be retrieved from the StorageBackend, e.g. some
/// network error. This mean that operation can be called again later of data can still be successfully
/// retrieved from another equivalent backend.
///
/// All logical errors, e.g. trying opening directory, should be reflected in the inner type, like OpenResponse.
/// Those variants are hidden inside Ok value because they should not trigger retrying operation.
pub trait StorageBackend {
    fn readdir(&self, path: &Path) -> Result<Vec<PathBuf>, StorageBackendError>;
    fn getattr(&self, path: &Path) -> Result<Option<Stat>, StorageBackendError>;
    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError>;
}
