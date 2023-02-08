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

mod descriptors;
pub mod models;
pub mod s3;

use std::path::Path;
use std::rc::Rc;

pub use descriptors::CloseOnDropDescriptor;
use wildland_corex::Storage;

pub use self::descriptors::OpenedFileDescriptor;
use self::models::{
    CreateDirResponse,
    GetattrResponse,
    OpenResponse,
    ReaddirResponse,
    RemoveDirResponse,
    RemoveFileResponse,
    StorageBackendError,
};

/// Error represents scenario when data could not be retrieved from the StorageBackend, e.g. some
/// network error. This mean that operation can be called again later of data can still be successfully
/// retrieved from another equivalent backend.
///
/// All logical errors, e.g. trying opening directory, should be reflected in the inner type, like OpenResponse.
/// Those variants are hidden inside Ok value because they should not trigger retrying operation.
pub trait StorageBackend {
    fn read_dir(&self, path: &Path) -> Result<ReaddirResponse, StorageBackendError>;
    fn metadata(&self, path: &Path) -> Result<GetattrResponse, StorageBackendError>;
    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError>;
    fn create_dir(&self, path: &Path) -> Result<CreateDirResponse, StorageBackendError>;
    fn remove_dir(&self, path: &Path) -> Result<RemoveDirResponse, StorageBackendError>;
    fn path_exists(&self, path: &Path) -> Result<bool, StorageBackendError>;
    fn remove_file(&self, path: &Path) -> Result<RemoveFileResponse, StorageBackendError>;
}

pub trait StorageBackendFactory {
    fn init_backend(&self, storage: Storage) -> anyhow::Result<Rc<dyn StorageBackend>>;
}
