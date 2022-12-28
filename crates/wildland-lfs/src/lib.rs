mod template;

use std::{
    fs::read_dir,
    path::{Path, PathBuf},
    rc::Rc,
};

use template::LocalFilesystemStorageTemplate;
use wildland_dfs::{storage_backend::StorageBackend, unencrypted::StorageBackendFactory, Storage};

#[derive(Debug)]
pub struct LocalFilesystemStorage {
    base_dir: PathBuf,
}

impl StorageBackend for LocalFilesystemStorage {
    fn readdir(&self, path: &Path) -> Vec<PathBuf> {
        let relative_path = if path.is_absolute() {
            path.strip_prefix("/").unwrap()
        } else {
            path
        };
        println!("{:?}", self.base_dir.join(relative_path));
        read_dir(self.base_dir.join(relative_path))
            .map(|readdir| {
                readdir
                    .into_iter()
                    .map(|entry| {
                        Path::new("/")
                            .join(entry.unwrap().path().strip_prefix(&self.base_dir).unwrap())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

pub struct LfsBackendFactory {}
impl StorageBackendFactory for LfsBackendFactory {
    fn init_backend(&self, storage: Storage) -> std::rc::Rc<dyn StorageBackend> {
        let template: LocalFilesystemStorageTemplate =
            serde_json::from_value(storage.data().clone()).unwrap(); // TODO unwrap
        Rc::new(LocalFilesystemStorage {
            base_dir: template.local_dir.join(template.container_prefix),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir, File},
        str::FromStr,
    };

    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use tempdir::TempDir;

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
        let backend = factory.init_backend(storage);

        create_dir(tmpdir.path().join("books")).unwrap(); // container dir
        let _ = File::create(tmpdir.path().join("books/file1")).unwrap();

        let files = backend.readdir(Path::new("/"));

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
        let backend = factory.init_backend(storage);

        create_dir(tmpdir.path().join("books")).unwrap(); // container dir
        create_dir(tmpdir.path().join("books").join("dir")).unwrap(); // container subdir
        let _ = File::create(tmpdir.path().join("books/dir/file1")).unwrap();

        let files = backend.readdir(Path::new("/dir"));

        assert_eq!(files, vec![PathBuf::from_str("/dir/file1").unwrap()]);
    }
}
