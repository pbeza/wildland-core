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

// Tests are divided into modules according to the used PathTranslator, which is a trait specifying the logic of translating
// absolute paths in terms of a user's namespace into exposed paths in a filesystem frontend. This logic handles such
// problems as conflicting paths in the user's namespace from different data sources.
mod uuid_dir_path_translator;

use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use rsfs::mem::{File, FS};
use rsfs::{DirEntry, FileType, GenFS, Metadata, OpenOptions};
use wildland_corex::dfs::interface::{DfsFrontendError, NodeType, Stat, UnixTimestamp};
use wildland_corex::{MockPathResolver, Storage};

use crate::storage_backends::{
    CloseError,
    GetattrResponse,
    OpenResponse,
    OpenedFileDescriptor,
    ReaddirResponse,
    SeekFrom,
    StorageBackendError,
};
use crate::unencrypted::{StorageBackend, StorageBackendFactory, UnencryptedDfs};

struct MufsAttrs {
    access_time: Option<UnixTimestamp>,
    modification_time: Option<UnixTimestamp>,
    change_time: Option<UnixTimestamp>,
    size: u64,
}

fn systime_to_unix(systime: SystemTime) -> UnixTimestamp {
    let timestamp = systime.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    UnixTimestamp {
        sec: timestamp.as_secs(),
        nano_sec: (timestamp.as_nanos() % 1_000_000_000) as u32,
    }
}

fn get_unix_time_of_file<T: AsRef<Path>, U: AsRef<FS>>(path: T, fs: U) -> MufsAttrs {
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

impl StorageBackend for Mufs {
    fn readdir(&self, path: &Path) -> Result<ReaddirResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);
        let file_type = self.fs.metadata(&path)?.file_type();
        if file_type.is_file() || file_type.is_symlink() {
            return Ok(ReaddirResponse::NotADirectory);
        }

        Ok(ReaddirResponse::Entries(
            self.fs
                .read_dir(path)?
                .map(|entry| {
                    Ok(Path::new("/").join(entry?.path().strip_prefix(&self.base_dir).unwrap()))
                })
                .collect::<Result<_, StorageBackendError>>()?,
        ))
    }

    fn getattr(&self, path: &Path) -> Result<GetattrResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        Ok(GetattrResponse::Found(
            self.fs
                .metadata(self.base_dir.join(relative_path))
                .map(|metadata| {
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
                        size: metadata.len(),
                        access_time: metadata.accessed().ok().map(systime_to_unix),
                        modification_time: metadata.modified().ok().map(systime_to_unix),
                        // NOTE: Mufs does not support ctime, for tests sake let's use creation time
                        change_time: metadata.created().ok().map(systime_to_unix),
                    }
                })?,
        ))
    }

    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);

        if !self.fs.metadata(&path).unwrap().is_file() {
            return Ok(OpenResponse::NotAFile);
        }

        let file = self.fs.new_openopts().read(true).write(true).open(path)?;

        let opened_file = MufsOpenedFile::new(file);

        Ok(OpenResponse::found(opened_file))
    }
}

#[derive(Debug)]
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

    fn seek(&mut self, seek_from: SeekFrom) -> Result<u64, DfsFrontendError> {
        Ok(self.inner.seek(seek_from.to_std())?)
    }
}

struct MufsFactory {
    fs: Rc<FS>,
}
impl MufsFactory {
    fn new(fs: Rc<FS>) -> Self {
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

type DfsFixture = (UnencryptedDfs, Rc<FS>);
fn dfs_with_fs(path_resolver: Box<MockPathResolver>) -> DfsFixture {
    let fs = Rc::new(FS::new());
    let factory = MufsFactory::new(fs.clone());
    let mut backend_factories: HashMap<String, Box<dyn StorageBackendFactory>> = HashMap::new();
    backend_factories.insert("MUFS".to_string(), Box::new(factory));

    let dfs = UnencryptedDfs::new(path_resolver, backend_factories);

    (dfs, fs)
}

pub fn new_mufs_storage(base_dir: impl Into<String>) -> Storage {
    Storage::new(
        Some("Test MUFS".to_owned()),
        "MUFS".to_owned(),
        serde_json::Value::String(base_dir.into()),
    )
}
