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

use crate::storage_backend::StorageBackend;
use std::{collections::HashMap, path::Path, rc::Rc};
use uuid::Uuid;
use wildland_corex::{
    dfs::interface::{DfsFrontend, NodeDescriptor},
    PathResolver, PathWithinStorage, Storage,
};

pub trait StorageBackendFactory {
    fn init_backend(&self, storage: Storage) -> Rc<dyn StorageBackend>;
}

pub struct UnencryptedDfs {
    path_resolver: Rc<dyn PathResolver>,
    storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    storage_backends: HashMap<Uuid, Rc<dyn StorageBackend>>, // Key: Storage uuid, Value: corresponding backend handling communication
}

impl UnencryptedDfs {
    pub fn new(
        path_resolver: Rc<dyn PathResolver>,
        storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    ) -> Self {
        Self {
            path_resolver,
            storage_backend_factories,
            storage_backends: HashMap::new(),
        }
    }

    fn get_backend(&mut self, storage: &Storage) -> Rc<dyn StorageBackend> {
        // TODO unwrap
        // TODO casing
        self.storage_backends
            .entry(storage.uuid())
            .or_insert_with(|| {
                self.storage_backend_factories
                    .get(&storage.backend_type().to_uppercase())
                    .unwrap()
                    .init_backend(storage.clone())
            })
            .clone()
    }
}

impl DfsFrontend for UnencryptedDfs {
    fn readdir<P: AsRef<Path>>(&mut self, path: P) -> Vec<NodeDescriptor> {
        let resolved_paths = self.path_resolver.resolve(path.as_ref());
        let nodes = resolved_paths
            .into_iter()
            .flat_map(|PathWithinStorage { path, storages }| {
                let mut backends = storages
                    .into_iter()
                    .map(|storage| (self.get_backend(&storage), storage));
                // TODO getting first should be a temporary policy, maybe we should ping backends to check if any of them
                // is responsive and use the one that answered as the first one.
                let (backend, storage) = backends.next().unwrap(); // TODO return error
                backend
                    .readdir(&path)
                    .into_iter()
                    .map(move |path| NodeDescriptor {
                        storage: storage.clone(),
                        path,
                    })
            });
        // TODO check conflicts: more than one container may have nodes claiming the same path
        nodes.collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::*;
    use mockall::predicate;
    use pretty_assertions::assert_eq;
    use rsfs::{mem::FS, DirEntry, GenFS};
    use rstest::{fixture, rstest};
    use wildland_corex::MockPathResolver;

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
        fn readdir(&self, path: &Path) -> Vec<PathBuf> {
            let relative_path = if path.is_absolute() {
                path.strip_prefix("/").unwrap()
            } else {
                path
            };
            self.fs
                .read_dir(self.base_dir.join(relative_path))
                .and_then(|readdir| {
                    Ok(readdir
                        .into_iter()
                        .map(|entry| {
                            Path::new("/")
                                .join(entry.unwrap().path().strip_prefix(&self.base_dir).unwrap())
                                .into()
                        })
                        .collect())
                })
                .unwrap_or_default()
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
        fn init_backend(&self, storage: Storage) -> Rc<dyn StorageBackend> {
            Rc::new(Mufs::new(
                self.fs.clone(),
                serde_json::from_value::<String>(storage.data().clone()).unwrap(),
            ))
        }
    }

    type DfsFixture = (UnencryptedDfs, Rc<MockPathResolver>, Rc<FS>);
    #[fixture]
    fn dfs_with_path_resolver_and_fs() -> DfsFixture {
        let path_resolver = MockPathResolver::new();
        let path_resolver = Rc::new(path_resolver);

        let fs = Rc::new(FS::new());
        let factory = MufsFactory::new(fs.clone());
        let mut backend_factories: HashMap<String, Box<dyn StorageBackendFactory>> = HashMap::new();
        backend_factories.insert("MUFS".to_string(), Box::new(factory));

        let dfs = UnencryptedDfs::new(path_resolver.clone(), backend_factories);

        (dfs, path_resolver, fs)
    }

    fn new_mufs_storage(base_dir: impl Into<String>) -> Storage {
        Storage::new(
            Some("Test MUFS".to_owned()),
            "MUFS".to_owned(),
            serde_json::Value::String(base_dir.into()),
        )
    }

    #[rstest]
    fn test_listing_files_from_root_of_one_container(dfs_with_path_resolver_and_fs: DfsFixture) {
        let (mut dfs, path_resolver, fs) = dfs_with_path_resolver_and_fs;
        let mufs_storage = new_mufs_storage("/");

        unsafe {
            (Rc::as_ptr(&path_resolver) as *mut MockPathResolver)
                .as_mut()
                .unwrap()
                .expect_resolve()
                .with(predicate::eq(Path::new("/a/b/")))
                .times(2)
                .returning({
                    let storage = mufs_storage.clone();
                    move |_path| {
                        vec![PathWithinStorage {
                            path: "/".into(),
                            storages: vec![storage.clone()],
                        }]
                    }
                });
        }

        let files_descriptors = dfs.readdir("/a/b/");
        assert_eq!(files_descriptors, vec![]);

        fs.create_file("/file_in_root").unwrap();
        let files_descriptors = dfs.readdir("/a/b/");
        assert_eq!(
            files_descriptors,
            vec![NodeDescriptor {
                storage: mufs_storage.clone(),
                path: PathBuf::from_str("/file_in_root").unwrap()
            }]
        );
    }

    #[rstest]
    fn test_listing_files_from_nested_dir_of_one_container(
        dfs_with_path_resolver_and_fs: DfsFixture,
    ) {
        let (mut dfs, path_resolver, fs) = dfs_with_path_resolver_and_fs;
        let mufs_storage = new_mufs_storage("/");

        unsafe {
            (Rc::as_ptr(&path_resolver) as *mut MockPathResolver)
                .as_mut()
                .unwrap()
                .expect_resolve()
                .with(predicate::eq(Path::new("/a/b/dir")))
                .times(2)
                .returning({
                    let storage = mufs_storage.clone();
                    move |_path| {
                        vec![PathWithinStorage {
                            path: "/dir".into(),
                            storages: vec![storage.clone()],
                        }]
                    }
                })
        };

        let files_descriptors = dfs.readdir("/a/b/dir");
        assert_eq!(files_descriptors, vec![]);

        fs.create_dir("/dir/").unwrap();
        fs.create_file("/dir/nested_file_1").unwrap();
        fs.create_file("/dir/nested_file_2").unwrap();

        let files_descriptors = dfs.readdir("/a/b/dir");
        assert_eq!(
            files_descriptors,
            vec![
                NodeDescriptor {
                    storage: mufs_storage.clone(),
                    path: PathBuf::from_str("/dir/nested_file_1").unwrap()
                },
                NodeDescriptor {
                    storage: mufs_storage.clone(),
                    path: PathBuf::from_str("/dir/nested_file_2").unwrap()
                }
            ]
        );
    }

    #[rstest]
    fn test_listing_dirs_from_one_container(dfs_with_path_resolver_and_fs: DfsFixture) {
        let (mut dfs, path_resolver, fs) = dfs_with_path_resolver_and_fs;
        let mufs_storage = new_mufs_storage("/");

        unsafe {
            (Rc::as_ptr(&path_resolver) as *mut MockPathResolver)
                .as_mut()
                .unwrap()
                .expect_resolve()
                .with(predicate::eq(Path::new("/")))
                .times(2)
                .returning({
                    let storage = mufs_storage.clone();
                    move |_path| {
                        vec![PathWithinStorage {
                            path: "/".into(),
                            storages: vec![storage.clone()],
                        }]
                    }
                })
        };

        let files_descriptors = dfs.readdir("/");
        assert_eq!(files_descriptors, vec![]);

        fs.create_dir("/dir_a").unwrap();
        fs.create_dir("/dir_b").unwrap();

        let files_descriptors = dfs.readdir("/");
        assert_eq!(
            files_descriptors,
            vec![
                NodeDescriptor {
                    storage: mufs_storage.clone(),
                    path: PathBuf::from_str("/dir_a").unwrap()
                },
                NodeDescriptor {
                    storage: mufs_storage.clone(),
                    path: PathBuf::from_str("/dir_b").unwrap()
                },
            ]
        );
    }

    #[rstest]
    fn test_listing_files_and_dirs_from_two_containers(dfs_with_path_resolver_and_fs: DfsFixture) {
        let (mut dfs, path_resolver, fs) = dfs_with_path_resolver_and_fs;
        // each container has its own subfolder
        let storage1 = new_mufs_storage("/storage1/");
        let storage2 = new_mufs_storage("/storage2/");

        unsafe {
            (Rc::as_ptr(&path_resolver) as *mut MockPathResolver)
                .as_mut()
                .unwrap()
                .expect_resolve()
                .with(predicate::eq(Path::new("/a/b/c/dir")))
                .times(2)
                .returning({
                    let storage1 = storage1.clone();
                    let storage2 = storage2.clone();
                    move |_path| {
                        vec![
                            PathWithinStorage {
                                path: "/dir".into(), // returned by a container claiming path `/a/b/c/`
                                storages: vec![storage1.clone()],
                            },
                            PathWithinStorage {
                                path: "/c/dir".into(), // returned by a container claiming path `/a/b/`
                                storages: vec![storage2.clone()],
                            },
                        ]
                    }
                })
        };

        fs.create_dir("/storage1/").unwrap();
        fs.create_dir("/storage1/dir/").unwrap();
        fs.create_dir("/storage2/").unwrap();
        fs.create_dir("/storage2/c/").unwrap();
        fs.create_dir("/storage2/c/dir/").unwrap();

        let files_descriptors = dfs.readdir("/a/b/c/dir");
        assert_eq!(files_descriptors, vec![]);

        fs.create_file("/storage1/dir/file_from_container_1")
            .unwrap();
        fs.create_dir("/storage2/c/dir/next_dir").unwrap();
        fs.create_file("/storage2/c/dir/file_from_container_2")
            .unwrap();

        let files_descriptors = dfs.readdir("/a/b/c/dir");
        assert_eq!(
            files_descriptors,
            vec![
                NodeDescriptor {
                    storage: storage1.clone(),
                    path: PathBuf::from_str("/dir/file_from_container_1").unwrap()
                },
                NodeDescriptor {
                    storage: storage2.clone(),
                    path: PathBuf::from_str("/c/dir/file_from_container_2").unwrap()
                },
                NodeDescriptor {
                    storage: storage2.clone(),
                    path: PathBuf::from_str("/c/dir/next_dir").unwrap()
                }
            ]
        );
    }

    // TODO test many storages for one container
    // TODO test scenarios when more than one node would claim the same path
}
