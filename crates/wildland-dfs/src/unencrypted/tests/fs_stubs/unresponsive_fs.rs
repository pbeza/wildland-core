use std::path::Path;
use std::rc::Rc;

use anyhow::anyhow;
use wildland_corex::dfs::interface::{
    DfsFrontendError,
    FsStat,
    Stat,
    UnixTimestamp,
    WlPermissions,
};
use wildland_corex::Storage;

use crate::storage_backends::models::{
    CloseError,
    CreateDirResponse,
    CreateFileResponse,
    MetadataResponse,
    OpenResponse,
    ReadDirResponse,
    RemoveDirResponse,
    RemoveFileResponse,
    RenameResponse,
    SeekFrom,
    SetPermissionsResponse,
    StatFsResponse,
    StorageBackendError,
};
use crate::storage_backends::OpenedFileDescriptor;
use crate::unencrypted::{StorageBackend, StorageBackendFactory};

fn storage_backend_error() -> StorageBackendError {
    StorageBackendError::Generic {
        backend_type: "UnresponsiveFs".into(),
        inner: anyhow!("Unresponsive"),
    }
}

struct UnresponsiveFs {}

impl StorageBackend for UnresponsiveFs {
    fn read_dir(&self, _: &Path) -> Result<ReadDirResponse, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn metadata(&self, _: &Path) -> Result<MetadataResponse, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn open(&self, _: &Path) -> Result<OpenResponse, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn create_dir(&self, _: &Path) -> Result<CreateDirResponse, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn remove_dir(&self, _: &Path) -> Result<RemoveDirResponse, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn path_exists(&self, _: &Path) -> Result<bool, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn remove_file(&self, _: &Path) -> Result<RemoveFileResponse, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn create_file(&self, _: &Path) -> Result<CreateFileResponse, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn rename(&self, _: &Path, _: &Path) -> Result<RenameResponse, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn set_permissions(
        &self,
        _: &Path,
        _: WlPermissions,
    ) -> Result<SetPermissionsResponse, StorageBackendError> {
        Err(storage_backend_error())
    }

    fn stat_fs(&self, _: &Path) -> Result<StatFsResponse, StorageBackendError> {
        Err(storage_backend_error())
    }
}

pub struct UnresponsiveFsFile {}

impl OpenedFileDescriptor for UnresponsiveFsFile {
    fn close(&self) -> Result<(), CloseError> {
        Ok(())
    }

    fn read(&mut self, _: usize) -> Result<Vec<u8>, DfsFrontendError> {
        Err(DfsFrontendError::StorageNotResponsive)
    }

    fn write(&mut self, _: &[u8]) -> Result<usize, DfsFrontendError> {
        Err(DfsFrontendError::StorageNotResponsive)
    }

    fn seek(&mut self, _: SeekFrom) -> Result<usize, DfsFrontendError> {
        Err(DfsFrontendError::StorageNotResponsive)
    }

    fn set_permissions(&mut self, _: WlPermissions) -> Result<(), DfsFrontendError> {
        Err(DfsFrontendError::StorageNotResponsive)
    }

    fn sync(&mut self) -> Result<(), DfsFrontendError> {
        Err(DfsFrontendError::StorageNotResponsive)
    }

    fn metadata(&mut self) -> Result<Stat, DfsFrontendError> {
        Err(DfsFrontendError::StorageNotResponsive)
    }

    fn set_times(
        &mut self,
        _: Option<UnixTimestamp>,
        _: Option<UnixTimestamp>,
    ) -> Result<(), DfsFrontendError> {
        Err(DfsFrontendError::StorageNotResponsive)
    }

    fn set_length(&mut self, _: usize) -> Result<(), DfsFrontendError> {
        Err(DfsFrontendError::StorageNotResponsive)
    }

    fn stat_fs(&mut self) -> Result<FsStat, DfsFrontendError> {
        Err(DfsFrontendError::StorageNotResponsive)
    }
}

pub struct UnresponsiveFsFactory {}

impl StorageBackendFactory for UnresponsiveFsFactory {
    fn init_backend(&self, _: Storage) -> anyhow::Result<Rc<dyn StorageBackend>> {
        Ok(Rc::new(UnresponsiveFs {}))
    }
}

pub fn new_unresponsive_fs_storage() -> Storage {
    Storage::new(
        Some("Test UnresponsiveFs".to_owned()),
        "UnresponsiveFs".to_owned(),
        serde_json::Value::Null,
    )
}
