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
use wildland_dfs::close_on_drop_descriptor::CloseOnDropDescriptor;
use wildland_dfs::storage_backend::{
    CloseError,
    OpenResponse,
    OpenedFileDescriptor,
    ReaddirResponse,
    SeekFrom,
    StorageBackend,
    StorageBackendError,
};
use wildland_dfs::unencrypted::StorageBackendFactory;
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
    fn readdir(&self, path: &Path) -> Result<ReaddirResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if path.is_file() || path.is_symlink() {
            Ok(ReaddirResponse::NotADirectory)
        } else {
            Ok(ReaddirResponse::Entries(
                fs::read_dir(path)?
                    .into_iter()
                    .map(|entry_result| {
                        Ok(Path::new("/").join(entry_result?.path().strip_prefix(&self.base_dir)?))
                    })
                    .collect::<Result<_, StorageBackendError>>()?,
            ))
        }
    }

    fn getattr(&self, path: &Path) -> Result<Option<Stat>, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if !path.exists() {
            return Ok(None);
        }

        Ok(fs::metadata(path).map(|metadata| {
            let file_type = metadata.file_type();
            Some(Stat {
                node_type: if file_type.is_file() {
                    NodeType::File
                } else if file_type.is_dir() {
                    NodeType::Dir
                } else if file_type.is_symlink() {
                    NodeType::Symlink
                } else {
                    return None;
                },
                size: metadata.len(),
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

            Ok(OpenResponse::Found(CloseOnDropDescriptor::new(Box::new(
                opened_file,
            ))))
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

    fn seek(&mut self, seek_from: SeekFrom) -> Result<u64, DfsFrontendError> {
        Ok(self.inner.seek(seek_from_to_std(seek_from))?)
    }
}

fn seek_from_to_std(seek_from: SeekFrom) -> std::io::SeekFrom {
    match seek_from {
        SeekFrom::Start(p) => std::io::SeekFrom::Start(p),
        SeekFrom::End(p) => std::io::SeekFrom::End(p),
        SeekFrom::Current(p) => std::io::SeekFrom::Current(p),
    }
}

pub struct LfsBackendFactory {}
impl StorageBackendFactory for LfsBackendFactory {
    fn init_backend(&self, storage: Storage) -> Result<Rc<dyn StorageBackend>, anyhow::Error> {
        let template: LocalFilesystemStorageTemplate =
            serde_json::from_value(storage.data().clone())?;
        Ok(Rc::new(LocalFilesystemStorage {
            base_dir: template.local_dir.join(template.container_prefix),
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
                "container_prefix": "books"
            }),
        );
        let factory = LfsBackendFactory {};
        let backend = factory.init_backend(storage).unwrap();

        create_dir(tmpdir.path().join("books")).unwrap(); // container dir
        let _ = File::create(tmpdir.path().join("books/file1")).unwrap();

        let files = backend.readdir(Path::new("/")).unwrap();

        assert_eq!(
            files,
            ReaddirResponse::Entries(vec![PathBuf::from_str("/file1").unwrap()])
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
                "container_prefix": "books"
            }),
        );
        let factory = LfsBackendFactory {};
        let backend = factory.init_backend(storage).unwrap();

        create_dir(tmpdir.path().join("books")).unwrap(); // container dir
        create_dir(tmpdir.path().join("books").join("dir")).unwrap(); // container subdir
        let _ = File::create(tmpdir.path().join("books/dir/file1")).unwrap();

        let files = backend.readdir(Path::new("/dir")).unwrap();

        assert_eq!(
            files,
            ReaddirResponse::Entries(vec![PathBuf::from_str("/dir/file1").unwrap()])
        );
    }
}
