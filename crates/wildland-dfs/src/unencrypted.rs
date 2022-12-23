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
use wildland_corex::{
    dfs::interface::{DfsFrontend, NodeDescriptor},
    PathResolver, PathWithinStorage, Storage,
};

pub struct UnencryptedDfs {
    path_resolver: Rc<dyn PathResolver>,
    // Assumed one backend instance for every Storage type, if there is a need of some connection
    // pool or a queue it should be handled internally by a backend object.
    storage_backends: HashMap<String, Rc<dyn StorageBackend>>,
}

impl UnencryptedDfs {
    pub fn new(path_resolver: Rc<dyn PathResolver>) -> Self {
        Self {
            path_resolver,
            storage_backends: HashMap::new(),
        }
    }

    fn get_backend(&mut self, storage: &Storage) -> Rc<dyn StorageBackend> {
        // TODO should it be case sensitive?

        #[cfg(test)]
        if storage.backend_type().to_uppercase().as_str() == "MUFS" {
            return self.storage_backends.get("MUFS").unwrap().clone(); // don't init MUFS cause it should be filled with nodes in tests
        }

        match storage.backend_type().to_uppercase().as_str() {
            // TODO init backend if needed
            _ => todo!(),
        }
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
        fs: FS,
    }
    impl Mufs {
        fn new() -> Self {
            Self { fs: FS::new() }
        }
        fn create_file(self: &Rc<Self>, path: impl AsRef<Path>) {
            unsafe {
                (Rc::as_ptr(&self) as *mut Mufs)
                    .as_mut()
                    .unwrap()
                    .fs
                    .create_file(path)
                    .unwrap();
            }
        }
        fn create_dir(self: &Rc<Self>, path: impl AsRef<Path>) {
            unsafe {
                (Rc::as_ptr(&self) as *mut Mufs)
                    .as_mut()
                    .unwrap()
                    .fs
                    .create_dir(path)
                    .unwrap();
            }
        }
    }
    impl StorageBackend for Mufs {
        fn readdir(&self, path: &Path) -> Vec<PathBuf> {
            self.fs
                .read_dir(path)
                .and_then(|readdir| {
                    Ok(readdir
                        .into_iter()
                        .map(|entry| entry.unwrap().path())
                        .collect())
                })
                .unwrap_or_default()
        }
    }

    type DfsFixture = (UnencryptedDfs, Rc<MockPathResolver>, Rc<Mufs>);
    #[fixture]
    fn dfs_with_path_resolver_and_mufs() -> DfsFixture {
        let path_resolver = MockPathResolver::new();
        let path_resolver = Rc::new(path_resolver);

        let mut dfs = UnencryptedDfs::new(path_resolver.clone());
        let mufs = Rc::new(Mufs::new());
        dfs.storage_backends
            .insert("MUFS".to_string(), mufs.clone());
        (dfs, path_resolver, mufs)
    }

    #[fixture]
    fn mufs_storage() -> Storage {
        Storage::new(
            Some("Test MUFS".to_owned()),
            "MUFS".to_owned(),
            serde_json::Value::Null,
        )
    }

    #[fixture]
    fn another_mufs_storage(mufs_storage: Storage) -> Storage {
        mufs_storage
    }

    #[rstest]
    fn test_listing_files_from_root_of_one_container(
        dfs_with_path_resolver_and_mufs: DfsFixture,
        mufs_storage: Storage,
    ) {
        let (mut dfs, path_resolver, mufs) = dfs_with_path_resolver_and_mufs;

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

        mufs.create_file("/file_in_root");
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
        dfs_with_path_resolver_and_mufs: (impl DfsFrontend, Rc<MockPathResolver>, Rc<Mufs>),
        mufs_storage: Storage,
    ) {
        let (mut dfs, path_resolver, mufs) = dfs_with_path_resolver_and_mufs;

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

        mufs.create_dir("/dir/");
        mufs.create_file("/dir/nested_file_1");
        mufs.create_file("/dir/nested_file_2");

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
    fn test_listing_dirs_from_one_container(
        dfs_with_path_resolver_and_mufs: (impl DfsFrontend, Rc<MockPathResolver>, Rc<Mufs>),
        mufs_storage: Storage,
    ) {
        let (mut dfs, path_resolver, mufs) = dfs_with_path_resolver_and_mufs;

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

        mufs.create_dir("/dir_a");
        mufs.create_dir("/dir_b");

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
    fn test_listing_files_and_dirs_from_two_containers(
        dfs_with_path_resolver_and_mufs: (impl DfsFrontend, Rc<MockPathResolver>, Rc<Mufs>),
        mufs_storage: Storage,
        another_mufs_storage: Storage,
    ) {
        let (mut dfs, path_resolver, mufs) = dfs_with_path_resolver_and_mufs;

        unsafe {
            (Rc::as_ptr(&path_resolver) as *mut MockPathResolver)
                .as_mut()
                .unwrap()
                .expect_resolve()
                .with(predicate::eq(Path::new("/a/b/c/dir")))
                .times(2)
                .returning({
                    let storage = mufs_storage.clone();
                    let another_mufs_storage = another_mufs_storage.clone();
                    move |_path| {
                        vec![
                            PathWithinStorage {
                                path: "/dir".into(), // returned by a container claiming path `/a/b/c/`
                                storages: vec![storage.clone()],
                            },
                            PathWithinStorage {
                                path: "/c/dir".into(), // returned by a container claiming path `/a/b/`
                                storages: vec![another_mufs_storage.clone()],
                            },
                        ]
                    }
                })
        };

        mufs.create_dir("/dir/");
        mufs.create_dir("/c/");
        mufs.create_dir("/c/dir/");

        let files_descriptors = dfs.readdir("/a/b/c/dir");
        assert_eq!(files_descriptors, vec![]);

        mufs.create_dir("/c/dir/next_dir"); // TODO handle type in file descriptor
        mufs.create_file("/dir/file_from_container_1");
        mufs.create_file("/c/dir/file_from_container_2");

        let files_descriptors = dfs.readdir("/a/b/c/dir");
        assert_eq!(
            files_descriptors,
            vec![
                NodeDescriptor {
                    storage: mufs_storage.clone(),
                    path: PathBuf::from_str("/dir/file_from_container_1").unwrap()
                },
                NodeDescriptor {
                    storage: another_mufs_storage.clone(),
                    path: PathBuf::from_str("/c/dir/file_from_container_2").unwrap()
                },
                NodeDescriptor {
                    storage: another_mufs_storage.clone(),
                    path: PathBuf::from_str("/c/dir/next_dir").unwrap()
                }
            ]
        );
    }
}
