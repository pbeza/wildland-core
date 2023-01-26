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

use std::fs;
use std::os::unix::prelude::MetadataExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use template::LocalFilesystemStorageTemplate;
use wildland_dfs::storage_backend::{StorageBackend, StorageBackendError};
use wildland_dfs::unencrypted::StorageBackendFactory;
use wildland_dfs::{NodeType, Stat, Storage, UnixTimestamp};

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
    fn readdir(&self, path: &Path) -> Result<Vec<PathBuf>, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if path.is_file() || path.is_symlink() {
            Err(StorageBackendError::NotADirectory)
        } else {
            fs::read_dir(path)?
                .into_iter()
                .map(|entry_result| {
                    Ok(Path::new("/").join(entry_result?.path().strip_prefix(&self.base_dir)?))
                })
                .collect()
        }
    }

    fn getattr(&self, path: &Path) -> Result<Option<Stat>, StorageBackendError> {
        let relative_path = strip_root(path);

        Ok(
            fs::metadata(self.base_dir.join(relative_path)).map(|metadata| {
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
            })?,
        )
    }

    fn open(
        &self,
        path: &Path,
    ) -> Result<Option<wildland_dfs::FileDescriptor>, StorageBackendError> {
        todo!() // TODO implement it
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

        assert_eq!(files, vec![PathBuf::from_str("/file1").unwrap()]);
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

        assert_eq!(files, vec![PathBuf::from_str("/dir/file1").unwrap()]);
    }
}
