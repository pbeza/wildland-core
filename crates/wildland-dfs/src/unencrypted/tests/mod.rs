mod getattr;
mod readdir;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use rsfs::mem::FS;
use rsfs::{DirEntry, FileType, GenFS, Metadata};
use wildland_corex::dfs::interface::{NodeType, Stat, UnixTimestamp};
use wildland_corex::{MockPathResolver, Storage};

use crate::storage_backend::StorageBackendError;
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
    fn readdir(&self, path: &Path) -> Result<Vec<PathBuf>, StorageBackendError> {
        let relative_path = strip_root(path);
        let path = self.base_dir.join(relative_path);
        let file_type = self.fs.metadata(&path)?.file_type();
        if file_type.is_file() || file_type.is_symlink() {
            return Err(StorageBackendError::NotADirectory);
        }

        self.fs
            .read_dir(path)?
            .into_iter()
            .map(|entry| {
                Ok(Path::new("/").join(entry?.path().strip_prefix(&self.base_dir).unwrap()))
            })
            .collect()
    }

    fn getattr(&self, path: &Path) -> Result<Option<Stat>, StorageBackendError> {
        let relative_path = strip_root(path);
        Ok(self
            .fs
            .metadata(self.base_dir.join(relative_path))
            .map(|metadata| {
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
                    access_time: metadata.accessed().ok().map(systime_to_unix),
                    modification_time: metadata.modified().ok().map(systime_to_unix),
                    // NOTE: Mufs does not support ctime, for tests sake let's use creation time
                    change_time: metadata.created().ok().map(systime_to_unix),
                })
            })?)
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
    fn init_backend(&self, storage: Storage) -> Result<Rc<dyn StorageBackend>, anyhow::Error> {
        Ok(Rc::new(Mufs::new(
            self.fs.clone(),
            serde_json::from_value::<String>(storage.data().clone())?,
        )))
    }
}

type DfsFixture = (UnencryptedDfs, Rc<FS>);
fn dfs_with_fs(path_resolver: Rc<MockPathResolver>) -> DfsFixture {
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
