use std::io::{ErrorKind, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use rsfs::mem::{File, Metadata, FS};
use rsfs::unix_ext::PermissionsExt;
use rsfs::{DirEntry, File as _, FileType, GenFS, Metadata as _, OpenOptions, Permissions};
use wildland_corex::dfs::interface::{
    DfsFrontendError,
    FsStat,
    NodeType,
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

pub struct MufsAttrs {
    pub access_time: Option<UnixTimestamp>,
    pub modification_time: Option<UnixTimestamp>,
    pub change_time: Option<UnixTimestamp>,
    pub size: u64,
}

fn systime_to_unix(systime: SystemTime) -> UnixTimestamp {
    let timestamp = systime.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    UnixTimestamp {
        sec: timestamp.as_secs(),
        nano_sec: (timestamp.as_nanos() % 1_000_000_000) as u32,
    }
}

pub fn get_unix_time_of_file<T: AsRef<Path>, U: AsRef<FS>>(path: T, fs: U) -> MufsAttrs {
    let md = fs.as_ref().metadata(path).unwrap();
    MufsAttrs {
        access_time: md.accessed().ok().map(systime_to_unix),
        modification_time: md.modified().ok().map(systime_to_unix),
        // NOTE: Mufs does not support ctime, for tests sake let's use creation time
        change_time: md.created().ok().map(systime_to_unix),
        size: md.len(),
    }
}

/// Made up Filesystem
struct Mufs {
    fs: Rc<FS>,
    base_dir: PathBuf,
}

impl Mufs {
    fn new(fs: Rc<FS>, base_dir: impl Into<PathBuf>) -> Self {
        Self {
            fs,
            base_dir: base_dir.into(),
        }
    }
}

fn strip_root(path: &Path) -> &Path {
    if path.is_absolute() {
        path.strip_prefix("/").unwrap()
    } else {
        path
    }
}

fn map_to_storage_backend_error(err: impl Into<anyhow::Error>) -> StorageBackendError {
    StorageBackendError::Generic {
        backend_type: "MUFS".into(),
        inner: err.into(),
    }
}

impl StorageBackend for Mufs {
    fn read_dir(&self, path: &Path) -> Result<ReadDirResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);
        let file_type = match self.fs.metadata(&path) {
            Ok(metadata) => metadata.file_type(),
            Err(_) => return Ok(ReadDirResponse::NoSuchPath),
        };

        if !file_type.is_dir() {
            return Ok(ReadDirResponse::NotADirectory);
        }

        Ok(ReadDirResponse::Entries(
            self.fs
                .read_dir(path)
                .map_err(map_to_storage_backend_error)?
                .map(|entry| {
                    Ok(Path::new("/").join(
                        entry
                            .map_err(map_to_storage_backend_error)?
                            .path()
                            .strip_prefix(&self.base_dir)
                            .unwrap(),
                    ))
                })
                .collect::<Result<_, StorageBackendError>>()?,
        ))
    }

    fn metadata(&self, path: &Path) -> Result<MetadataResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        Ok(MetadataResponse::Found(
            self.fs
                .metadata(self.base_dir.join(relative_path))
                .map(map_metadata_to_stat)
                .map_err(map_to_storage_backend_error)?,
        ))
    }

    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if !self.fs.metadata(&path).unwrap().is_file() {
            return Ok(OpenResponse::NotAFile);
        }

        let file = self
            .fs
            .new_openopts()
            .read(true)
            .write(true)
            .open(path)
            .map_err(map_to_storage_backend_error)?;

        let opened_file = MufsOpenedFile::new(file);

        Ok(OpenResponse::found(opened_file))
    }

    fn create_dir(&self, path: &Path) -> Result<CreateDirResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        match self.fs.create_dir(path) {
            Ok(()) => Ok(CreateDirResponse::Created),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(CreateDirResponse::InvalidParent),
                std::io::ErrorKind::AlreadyExists => Ok(CreateDirResponse::PathAlreadyExists),
                _ => Err(map_to_storage_backend_error(e)),
            },
        }
    }

    fn remove_dir(&self, path: &Path) -> Result<RemoveDirResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if path == Path::new("/") {
            return Ok(RemoveDirResponse::RootRemovalNotAllowed);
        }

        if let Ok(metadata) = self.fs.metadata(&path) {
            let file_type = metadata.file_type();
            if !file_type.is_dir() {
                return Ok(RemoveDirResponse::NotADirectory);
            }

            if self.fs.read_dir(&path).unwrap().next().is_some() {
                return Ok(RemoveDirResponse::DirNotEmpty);
            }

            Ok(self
                .fs
                .remove_dir(path)
                .map(|_| RemoveDirResponse::Removed)
                .map_err(map_to_storage_backend_error)?)
        } else {
            Ok(RemoveDirResponse::NotFound)
        }
    }

    fn path_exists(&self, path: &Path) -> Result<bool, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        Ok(self.fs.metadata(path).is_ok())
    }

    fn remove_file(&self, path: &Path) -> Result<RemoveFileResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if let Ok(metadata) = self.fs.metadata(&path) {
            let file_type = metadata.file_type();
            if !file_type.is_file() {
                return Ok(RemoveFileResponse::NotAFile);
            }

            match self.fs.remove_file(path) {
                Ok(_) => Ok(RemoveFileResponse::Removed),
                Err(e) => match e.kind() {
                    std::io::ErrorKind::NotFound => Ok(RemoveFileResponse::NotFound),
                    _ => Err(map_to_storage_backend_error(e)),
                },
            }
        } else {
            Ok(RemoveFileResponse::NotFound)
        }
    }

    fn create_file(&self, path: &Path) -> Result<CreateFileResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        match self.fs.create_file(path.as_path()) {
            Ok(_) => {
                let opened_file = self
                    .fs
                    .new_openopts()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .read(true)
                    .open(path)
                    .map_err(map_to_storage_backend_error)?;
                Ok(CreateFileResponse::created(MufsOpenedFile::new(
                    opened_file,
                )))
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(CreateFileResponse::InvalidParent),
                _ => Err(map_to_storage_backend_error(e)),
            },
        }
    }

    fn rename(
        &self,
        old_path: &Path,
        new_path: &Path,
    ) -> Result<RenameResponse, StorageBackendError> {
        let relative_old_path = strip_root(old_path);
        let old_path = self.base_dir.join(relative_old_path);

        let relative_new_path = strip_root(new_path);
        let new_path = self.base_dir.join(relative_new_path);

        if self.fs.metadata(&old_path).is_ok() {
            if self.fs.metadata(&new_path).is_ok() {
                Ok(RenameResponse::TargetPathAlreadyExists)
            } else {
                match new_path.strip_prefix(&old_path) {
                    Ok(_) => Ok(RenameResponse::SourceIsParentOfTarget),
                    Err(_) => match self.fs.rename(old_path, new_path) {
                        Ok(_) => Ok(RenameResponse::Renamed),
                        Err(e) => match e.kind() {
                            std::io::ErrorKind::NotFound => Ok(RenameResponse::NotFound),
                            _ => Err(map_to_storage_backend_error(e)),
                        },
                    },
                }
            }
        } else {
            Ok(RenameResponse::NotFound)
        }
    }

    fn set_permissions(
        &self,
        path: &Path,
        permissions: WlPermissions,
    ) -> Result<SetPermissionsResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        let mode = if permissions.is_readonly() {
            0o444
        } else {
            0o644
        };

        match self
            .fs
            .set_permissions(path, rsfs::mem::Permissions::from_mode(mode))
        {
            Ok(_) => Ok(SetPermissionsResponse::Set),
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(SetPermissionsResponse::NotFound),
                _ => Err(map_to_storage_backend_error(e)),
            },
        }
    }

    fn stat_fs(&self, _path: &Path) -> Result<StatFsResponse, StorageBackendError> {
        Ok(StatFsResponse::NotSupported(
            "MuFS does not support `stat_fs` operation".into(),
        ))
    }
}

pub struct MufsOpenedFile {
    inner: File,
}

impl MufsOpenedFile {
    fn new(inner: File) -> Self {
        Self { inner }
    }
}

impl OpenedFileDescriptor for MufsOpenedFile {
    fn close(&self) -> Result<(), CloseError> {
        // rsfs File is closed when going out of scope, so there is nothing to do here
        Ok(())
    }

    fn read(&mut self, count: usize) -> Result<Vec<u8>, DfsFrontendError> {
        let mut buffer = vec![0; count];
        let read_count = self.inner.read(&mut buffer)?;
        if read_count < buffer.len() {
            buffer.truncate(read_count);
        }
        Ok(buffer)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, DfsFrontendError> {
        Ok(self.inner.write(buf)?)
    }

    fn seek(&mut self, seek_from: SeekFrom) -> Result<usize, DfsFrontendError> {
        Ok(self.inner.seek(seek_from.to_std())? as _)
    }

    fn set_permissions(&mut self, permissions: WlPermissions) -> Result<(), DfsFrontendError> {
        let mode = if permissions.is_readonly() {
            0o444
        } else {
            0o644
        };
        Ok(self
            .inner
            .set_permissions(rsfs::mem::Permissions::from_mode(mode))?)
    }

    fn sync(&mut self) -> Result<(), DfsFrontendError> {
        Ok(self.inner.sync_all()?)
    }

    fn metadata(&mut self) -> Result<Stat, DfsFrontendError> {
        Ok(self.inner.metadata().map(map_metadata_to_stat)?)
    }

    fn set_times(
        &mut self,
        _access_time: Option<UnixTimestamp>,
        _modification_time: Option<UnixTimestamp>,
    ) -> Result<(), DfsFrontendError> {
        Err(DfsFrontendError::Generic(
            "`set_times` is not supported for MuFS".into(),
        ))
    }

    fn set_length(&mut self, length: usize) -> Result<(), DfsFrontendError> {
        Ok(self.inner.set_len(length as u64)?)
    }

    fn stat_fs(&mut self) -> Result<FsStat, DfsFrontendError> {
        Err(DfsFrontendError::Generic(
            "MuFS does not support `stat_fs` operation".into(),
        ))
    }
}

fn map_metadata_to_stat(metadata: Metadata) -> Stat {
    let file_type = metadata.file_type();
    Stat {
        node_type: if file_type.is_file() {
            NodeType::File
        } else if file_type.is_dir() {
            NodeType::Dir
        } else if file_type.is_symlink() {
            NodeType::Symlink
        } else {
            NodeType::Other
        },
        size: metadata.len() as _,
        access_time: metadata.accessed().ok().map(systime_to_unix),
        modification_time: metadata.modified().ok().map(systime_to_unix),
        // NOTE: Mufs does not support ctime, for tests sake let's use creation time
        change_time: metadata.created().ok().map(systime_to_unix),
        permissions: if metadata.permissions().readonly() {
            WlPermissions::readonly()
        } else {
            WlPermissions::read_write()
        },
    }
}

pub struct MufsFactory {
    fs: Rc<FS>,
}

impl MufsFactory {
    pub fn new(fs: Rc<FS>) -> Self {
        Self { fs }
    }
}

impl StorageBackendFactory for MufsFactory {
    fn init_backend(&self, storage: Storage) -> anyhow::Result<Rc<dyn StorageBackend>> {
        Ok(Rc::new(Mufs::new(
            self.fs.clone(),
            serde_json::from_value::<String>(storage.data())?,
        )))
    }
}

pub fn new_mufs_storage(base_dir: impl Into<String>) -> Storage {
    Storage::new(
        Some("Test MUFS".to_owned()),
        "MUFS".to_owned(),
        serde_json::Value::String(base_dir.into()),
    )
}
