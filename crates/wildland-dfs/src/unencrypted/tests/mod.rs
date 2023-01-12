mod getattr;
mod readdir;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rsfs::mem::FS;
use rsfs::{DirEntry, FileType, GenFS, Metadata};
use wildland_corex::dfs::interface::{NodeType, Stat};
use wildland_corex::{MockPathResolver, Storage};

use crate::storage_backend::StorageBackendError;
use crate::unencrypted::{StorageBackend, StorageBackendFactory, UnencryptedDfs};

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
impl StorageBackend for Mufs {
    fn readdir(&self, path: &Path) -> Result<Vec<PathBuf>, StorageBackendError> {
        // todo extract
        let relative_path = if path.is_absolute() {
            path.strip_prefix("/").unwrap()
        } else {
            path
        };
        self.fs
            .read_dir(self.base_dir.join(relative_path))
            .map_err(|err| {
                if err.kind() == ErrorKind::NotADirectory {
                    StorageBackendError::NotADirectory
                } else {
                    StorageBackendError::Generic(err.into())
                }
            })
            .map(|readdir| {
                readdir
                    .into_iter()
                    .map(|entry| {
                        Path::new("/")
                            .join(entry.unwrap().path().strip_prefix(&self.base_dir).unwrap())
                    })
                    .collect()
            })
    }

    fn getattr(&self, path: &Path) -> Result<Option<Stat>, StorageBackendError> {
        let relative_path = if path.is_absolute() {
            path.strip_prefix("/").unwrap()
        } else {
            path
        };
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

fn new_mufs_storage(base_dir: impl Into<String>) -> Storage {
    Storage::new(
        Some("Test MUFS".to_owned()),
        "MUFS".to_owned(),
        serde_json::Value::String(base_dir.into()),
    )
}
