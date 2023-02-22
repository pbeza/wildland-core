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

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::os::unix::prelude::MetadataExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

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
    StorageBackendError,
};
use wildland_dfs::storage_backends::{OpenedFileDescriptor, StorageBackend, StorageBackendFactory};
use wildland_dfs::{DfsFrontendError, NodeType, Stat, Storage, UnixTimestamp};

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
                fs::read_dir(path)?
                    .map(|entry_result| {
                        Ok(Path::new("/").join(entry_result?.path().strip_prefix(&self.base_dir)?))
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

        Ok(fs::metadata(path).map(|metadata| {
            let file_type = metadata.file_type();
            MetadataResponse::Found(Stat {
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
            })
        })?)
    }

    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if !path.exists() {
            Ok(OpenResponse::NotFound)
        } else if !path.metadata()?.is_file() {
            Ok(OpenResponse::NotAFile)
        } else {
            let file = OpenOptions::new().read(true).write(true).open(path)?;

            let opened_file = StdFsOpenedFile::new(file);

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
                _ => Err(StorageBackendError::Generic(e.into())),
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

            Ok(std::fs::remove_dir(&path).map(|_| RemoveDirResponse::Removed)?)
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
                    _ => Err(StorageBackendError::Generic(e.into())),
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
                    .open(path)?;
                Ok(CreateFileResponse::created(StdFsOpenedFile::new(
                    opened_file,
                )))
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(CreateFileResponse::InvalidParent),
                _ => Err(StorageBackendError::Generic(e.into())),
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
                            _ => Err(StorageBackendError::Generic(e.into())),
                        },
                    },
                }
            }
        } else {
            Ok(RenameResponse::NotFound)
        }
    }
}

#[derive(Debug)]
pub struct StdFsOpenedFile {
    inner: File,
}

impl StdFsOpenedFile {
    fn new(inner: File) -> Self {
        Self { inner }
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
