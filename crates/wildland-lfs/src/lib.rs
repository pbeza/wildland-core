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

pub mod template;

use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{ErrorKind, Read, Seek, Write};
use std::os::unix::prelude::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use filetime::{set_file_atime, set_file_mtime, FileTime};
use template::LocalFilesystemStorageTemplate;
use wildland_dfs::storage_backends::models::{
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
use wildland_dfs::storage_backends::{OpenedFileDescriptor, StorageBackend, StorageBackendFactory};
use wildland_dfs::{
    DfsFrontendError,
    FsStat,
    NodeType,
    Stat,
    Storage,
    UnixTimestamp,
    WlPermissions,
};

#[derive(Debug)]
pub struct LocalFilesystemStorage {
    base_dir: PathBuf,
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
        backend_type: "LocalFilesystem".into(),
        inner: err.into(),
    }
}

impl StorageBackend for LocalFilesystemStorage {
    fn read_dir(&self, path: &Path) -> Result<ReadDirResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        let file_type = match std::fs::metadata(&path) {
            Ok(metadata) => metadata.file_type(),
            Err(_) => return Ok(ReadDirResponse::NoSuchPath),
        };

        if !file_type.is_dir() {
            Ok(ReadDirResponse::NotADirectory)
        } else {
            Ok(ReadDirResponse::Entries(
                fs::read_dir(path)
                    .map_err(map_to_storage_backend_error)?
                    .map(|entry_result| {
                        Ok(Path::new("/").join(
                            entry_result
                                .map_err(map_to_storage_backend_error)?
                                .path()
                                .strip_prefix(&self.base_dir)
                                .map_err(map_to_storage_backend_error)?,
                        ))
                    })
                    .collect::<Result<_, StorageBackendError>>()?,
            ))
        }
    }

    fn metadata(&self, path: &Path) -> Result<MetadataResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if !path.exists() {
            return Ok(MetadataResponse::NotFound);
        }

        Ok(MetadataResponse::Found(
            fs::metadata(path)
                .map(map_metadata_to_stat)
                .map_err(map_to_storage_backend_error)?,
        ))
    }

    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if !path.exists() {
            Ok(OpenResponse::NotFound)
        } else if !path
            .metadata()
            .map_err(map_to_storage_backend_error)?
            .is_file()
        {
            Ok(OpenResponse::NotAFile)
        } else {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path.as_path())
                .map_err(map_to_storage_backend_error)?;

            let opened_file = StdFsOpenedFile::new(file, path);

            Ok(OpenResponse::found(opened_file))
        }
    }

    fn create_dir(&self, path: &Path) -> Result<CreateDirResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        match std::fs::create_dir(path) {
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

        if let Ok(metadata) = std::fs::metadata(&path) {
            let file_type = metadata.file_type();
            if !file_type.is_dir() {
                return Ok(RemoveDirResponse::NotADirectory);
            }

            if path.read_dir().unwrap().next().is_some() {
                return Ok(RemoveDirResponse::DirNotEmpty);
            }

            Ok(std::fs::remove_dir(&path)
                .map(|_| RemoveDirResponse::Removed)
                .map_err(map_to_storage_backend_error)?)
        } else {
            Ok(RemoveDirResponse::NotFound)
        }
    }

    fn path_exists(&self, path: &Path) -> Result<bool, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        Ok(path.exists())
    }

    fn remove_file(&self, path: &Path) -> Result<RemoveFileResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if let Ok(metadata) = std::fs::metadata(&path) {
            let file_type = metadata.file_type();
            if !file_type.is_file() {
                return Ok(RemoveFileResponse::NotAFile);
            }

            match std::fs::remove_file(path) {
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

    fn create_file(
        &self,
        path: &Path,
    ) -> Result<wildland_dfs::storage_backends::models::CreateFileResponse, StorageBackendError>
    {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        match std::fs::File::create(path.as_path()) {
            Ok(_file) => {
                let opened_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .read(true)
                    .open(path.as_path())
                    .map_err(map_to_storage_backend_error)?;
                Ok(CreateFileResponse::created(StdFsOpenedFile::new(
                    opened_file,
                    path,
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

        if old_path.exists() {
            if new_path.exists() {
                Ok(RenameResponse::TargetPathAlreadyExists)
            } else {
                match new_path.strip_prefix(&old_path) {
                    Ok(_) => Ok(RenameResponse::SourceIsParentOfTarget),
                    Err(_) => match std::fs::rename(old_path, new_path) {
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

        match std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)) {
            Ok(_) => Ok(SetPermissionsResponse::Set),
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Ok(SetPermissionsResponse::NotFound),
                _ => Err(map_to_storage_backend_error(e)),
            },
        }
    }

    fn stat_fs(&self, path: &Path) -> Result<StatFsResponse, StorageBackendError> {
        stat_fs(path).map(StatFsResponse::Stat)
    }
}

fn stat_fs(path: &Path) -> Result<FsStat, StorageBackendError> {
    let statfs = rustix::fs::statfs(path).map_err(map_to_storage_backend_error)?;
    Ok(FsStat {
        block_size: statfs.f_bsize as u64,
        io_size: None,
        blocks: statfs.f_blocks,
        free_blocks: statfs.f_blocks,
        blocks_available: statfs.f_blocks,
        nodes: statfs.f_blocks,
        name_length: statfs.f_blocks,
    })
}

#[derive(Debug)]
pub struct StdFsOpenedFile {
    inner: File,
    path: PathBuf,
}

impl StdFsOpenedFile {
    fn new(inner: File, path: PathBuf) -> Self {
        Self { inner, path }
    }
}

impl OpenedFileDescriptor for StdFsOpenedFile {
    fn close(&self) -> Result<(), CloseError> {
        // std::fs::File is closed when going out of scope, so there is nothing to do here
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
            .set_permissions(std::fs::Permissions::from_mode(mode))?)
    }

    fn sync(&mut self) -> Result<(), DfsFrontendError> {
        Ok(self.inner.sync_all()?)
    }

    fn metadata(&mut self) -> Result<Stat, DfsFrontendError> {
        Ok(self.inner.metadata().map(map_metadata_to_stat)?)
    }

    fn set_times(
        &mut self,
        access_time: Option<UnixTimestamp>,
        modification_time: Option<UnixTimestamp>,
    ) -> Result<(), DfsFrontendError> {
        if let Some(access_time) = access_time {
            set_file_atime(
                self.path.as_path(),
                FileTime::from_unix_time(access_time.sec() as i64, access_time.nano_sec()),
            )?;
        }
        if let Some(modification_time) = modification_time {
            set_file_mtime(
                self.path.as_path(),
                FileTime::from_unix_time(
                    modification_time.sec() as i64,
                    modification_time.nano_sec(),
                ),
            )?;
        }
        Ok(())
    }

    fn set_length(&mut self, length: usize) -> Result<(), DfsFrontendError> {
        Ok(self.inner.set_len(length as u64)?)
    }

    fn stat_fs(&mut self) -> Result<FsStat, DfsFrontendError> {
        stat_fs(&self.path).map_err(|e| {
            DfsFrontendError::Generic(format!("Could not retrieve filesystem stats: {e}"))
        })
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
        access_time: Some(UnixTimestamp {
            sec: metadata.atime() as u64,
            nano_sec: metadata.atime_nsec() as u32,
        }),
        modification_time: Some(UnixTimestamp {
            sec: metadata.mtime() as u64,
            nano_sec: metadata.mtime_nsec() as u32,
        }),
        change_time: Some(UnixTimestamp {
            sec: metadata.ctime() as u64,
            nano_sec: metadata.ctime_nsec() as u32,
        }),
        permissions: if metadata.permissions().readonly() {
            WlPermissions::readonly()
        } else {
            WlPermissions::read_write()
        },
    }
}

pub struct LfsBackendFactory {}
impl StorageBackendFactory for LfsBackendFactory {
    fn init_backend(&self, storage: Storage) -> anyhow::Result<Rc<dyn StorageBackend>> {
        let template: LocalFilesystemStorageTemplate = serde_json::from_value(storage.data())?;
        Ok(Rc::new(LocalFilesystemStorage {
            base_dir: template.local_dir.join(template.container_dir),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir, File};
    use std::str::FromStr;

    use pretty_assertions::assert_eq;
    use serde_json::json;
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_reading_file_in_root_of_lfs_backend() {
        let tmpdir = TempDir::new("lfs").unwrap(); // storage provider dir
        let storage = Storage::new(
            Some("Test LFS".to_owned()),
            "LFS".to_owned(),
            json!({
                "local_dir": tmpdir.path(),
                "container_dir": "books"
            }),
        );
        let factory = LfsBackendFactory {};
        let backend = factory.init_backend(storage).unwrap();

        create_dir(tmpdir.path().join("books")).unwrap(); // container dir
        let _ = File::create(tmpdir.path().join("books/file1")).unwrap();

        let files = backend.read_dir(Path::new("/")).unwrap();

        assert_eq!(
            files,
            ReadDirResponse::Entries(vec![PathBuf::from_str("/file1").unwrap()])
        );
    }
    #[test]
    fn test_reading_file_in_subdir_of_lfs_backend() {
        let tmpdir = TempDir::new("lfs").unwrap(); // storage provider dir
        let storage = Storage::new(
            Some("Test LFS".to_owned()),
            "LFS".to_owned(),
            json!({
                "local_dir": tmpdir.path(),
                "container_dir": "books"
            }),
        );
        let factory = LfsBackendFactory {};
        let backend = factory.init_backend(storage).unwrap();

        create_dir(tmpdir.path().join("books")).unwrap(); // container dir
        create_dir(tmpdir.path().join("books").join("dir")).unwrap(); // container subdir
        let _ = File::create(tmpdir.path().join("books/dir/file1")).unwrap();

        let files = backend.read_dir(Path::new("/dir")).unwrap();

        assert_eq!(
            files,
            ReadDirResponse::Entries(vec![PathBuf::from_str("/dir/file1").unwrap()])
        );
    }
}
