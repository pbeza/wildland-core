use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use mockall::predicate;
use pretty_assertions::assert_eq;
use rsfs::mem::FS;
use rsfs::{DirEntry, GenFS};
use rstest::{fixture, rstest};
use wildland_corex::dfs::interface::{DfsFrontend, NodeDescriptor, NodeStorage};
use wildland_corex::{MockPathResolver, PathWithStorages, Storage};

use crate::storage_backend::StorageBackend;
use crate::unencrypted::{StorageBackendFactory, UnencryptedDfs};

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
    fn readdir(&self, path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let relative_path = if path.is_absolute() {
            path.strip_prefix("/").unwrap()
        } else {
            path
        };
        Ok(self
            .fs
            .read_dir(self.base_dir.join(relative_path))
            .map(|readdir| {
                readdir
                    .into_iter()
                    .map(|entry| {
                        Path::new("/")
                            .join(entry.unwrap().path().strip_prefix(&self.base_dir).unwrap())
                    })
                    .collect()
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
    fn init_backend(
        &self,
        storage: Storage,
    ) -> Result<Rc<dyn StorageBackend>, Box<dyn std::error::Error>> {
        Ok(Rc::new(Mufs::new(
            self.fs.clone(),
            serde_json::from_value::<String>(storage.data().clone())?,
        )))
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
                    vec![PathWithStorages {
                        path_within_storage: "/".into(),
                        storages: Some(vec![storage.clone()]),
                    }]
                }
            });
    }

    let files_descriptors = dfs.readdir("/a/b/".to_string());
    assert_eq!(files_descriptors, vec![]);

    fs.create_file("/file_in_root").unwrap();
    let files_descriptors = dfs.readdir("/a/b/".to_string());
    assert_eq!(
        files_descriptors,
        vec![NodeDescriptor {
            storage: Some(NodeStorage::new(
                mufs_storage,
                PathBuf::from_str("/file_in_root").unwrap()
            )),
            absolute_path: PathBuf::from_str("/a/b/file_in_root").unwrap(),
        }]
    );
}

#[rstest]
fn test_listing_files_from_nested_dir_of_one_container(dfs_with_path_resolver_and_fs: DfsFixture) {
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
                    vec![PathWithStorages {
                        path_within_storage: "/dir".into(),
                        storages: Some(vec![storage.clone()]),
                    }]
                }
            })
    };

    let files_descriptors = dfs.readdir("/a/b/dir".to_string());
    assert_eq!(files_descriptors, vec![]);

    fs.create_dir("/dir/").unwrap();
    fs.create_file("/dir/nested_file_1").unwrap();
    fs.create_file("/dir/nested_file_2").unwrap();

    let files_descriptors = dfs.readdir("/a/b/dir".to_string());
    assert_eq!(
        files_descriptors,
        vec![
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    mufs_storage.clone(),
                    PathBuf::from_str("/dir/nested_file_1").unwrap()
                )),
                absolute_path: PathBuf::from_str("/a/b/dir/nested_file_1").unwrap(),
            },
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    mufs_storage,
                    PathBuf::from_str("/dir/nested_file_2").unwrap()
                )),
                absolute_path: PathBuf::from_str("/a/b/dir/nested_file_2").unwrap(),
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
                    vec![PathWithStorages {
                        path_within_storage: "/".into(),
                        storages: Some(vec![storage.clone()]),
                    }]
                }
            })
    };

    let files_descriptors = dfs.readdir("/".to_string());
    assert_eq!(files_descriptors, vec![]);

    fs.create_dir("/dir_a").unwrap();
    fs.create_dir("/dir_b").unwrap();

    let files_descriptors = dfs.readdir("/".to_string());
    assert_eq!(
        files_descriptors,
        vec![
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    mufs_storage.clone(),
                    PathBuf::from_str("/dir_a").unwrap()
                )),
                absolute_path: PathBuf::from_str("/dir_a").unwrap(),
            },
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    mufs_storage,
                    PathBuf::from_str("/dir_b").unwrap()
                )),
                absolute_path: PathBuf::from_str("/dir_b").unwrap(),
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
                        PathWithStorages {
                            path_within_storage: "/dir".into(), // returned by a container claiming path `/a/b/c/`
                            storages: Some(vec![storage1.clone()]),
                        },
                        PathWithStorages {
                            path_within_storage: "/c/dir".into(), // returned by a container claiming path `/a/b/`
                            storages: Some(vec![storage2.clone()]),
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

    let files_descriptors = dfs.readdir("/a/b/c/dir".to_string());
    assert_eq!(files_descriptors, vec![]);

    fs.create_file("/storage1/dir/file_from_container_1")
        .unwrap();
    fs.create_dir("/storage2/c/dir/next_dir").unwrap();
    fs.create_file("/storage2/c/dir/file_from_container_2")
        .unwrap();

    let files_descriptors = dfs.readdir("/a/b/c/dir".to_string());
    assert_eq!(
        files_descriptors,
        vec![
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    storage1,
                    PathBuf::from_str("/dir/file_from_container_1").unwrap()
                )),
                absolute_path: PathBuf::from_str("/a/b/c/dir/file_from_container_1").unwrap(),
            },
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    storage2.clone(),
                    PathBuf::from_str("/c/dir/file_from_container_2").unwrap()
                )),
                absolute_path: PathBuf::from_str("/a/b/c/dir/file_from_container_2").unwrap(),
            },
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    storage2,
                    PathBuf::from_str("/c/dir/next_dir").unwrap()
                )),
                absolute_path: PathBuf::from_str("/a/b/c/dir/next_dir").unwrap(),
            }
        ]
    );
}

#[rstest]
fn test_getting_one_file_descriptor_from_container_with_multiple_storages(
    dfs_with_path_resolver_and_fs: DfsFixture,
) {
    let (mut dfs, path_resolver, fs) = dfs_with_path_resolver_and_fs;
    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    unsafe {
        (Rc::as_ptr(&path_resolver) as *mut MockPathResolver)
            .as_mut()
            .unwrap()
            .expect_resolve()
            .with(predicate::eq(Path::new("/a")))
            .times(2)
            .returning({
                let storage1 = storage1.clone();
                let storage2 = storage2;
                move |_path| {
                    vec![PathWithStorages {
                        path_within_storage: "/a".into(), // returned by a container claiming path `/a/`
                        storages: Some(vec![storage1.clone(), storage2.clone()]),
                    }]
                }
            })
    };

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/a").unwrap();
    fs.create_dir("/storage2/").unwrap();
    fs.create_dir("/storage2/a").unwrap();

    let files_descriptors = dfs.readdir("/a".to_string());
    assert_eq!(files_descriptors, vec![]);

    fs.create_file("/storage1/a/b").unwrap();
    fs.create_file("/storage2/a/b").unwrap();

    let files_descriptors = dfs.readdir("/a".to_string());
    assert_eq!(
        files_descriptors,
        vec![NodeDescriptor {
            storage: Some(NodeStorage::new(
                storage1,
                PathBuf::from_str("/a/b").unwrap()
            )),
            absolute_path: PathBuf::from_str("/a/b").unwrap(),
        },]
    );
}

/// Full Path is a concatenation of a path claimed by a container with a path of the file inside the container.
#[rstest]
fn test_more_than_one_file_descriptor_claim_the_same_full_path(
    dfs_with_path_resolver_and_fs: DfsFixture,
) {
    let (mut dfs, path_resolver, fs) = dfs_with_path_resolver_and_fs;
    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    unsafe {
        (Rc::as_ptr(&path_resolver) as *mut MockPathResolver)
            .as_mut()
            .unwrap()
            .expect_resolve()
            .with(predicate::eq(Path::new("/a/b/")))
            .times(2)
            .returning({
                let storage1 = storage1.clone();
                let storage2 = storage2.clone();
                move |_path| {
                    vec![
                        PathWithStorages {
                            path_within_storage: "/b/".into(), // returned by the container claiming path `/a/`
                            storages: Some(vec![storage1.clone()]),
                        },
                        PathWithStorages {
                            path_within_storage: "/".into(), // returned by the container claiming path `/a/b/`
                            storages: Some(vec![storage2.clone()]),
                        },
                    ]
                }
            })
    };

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/b").unwrap();
    fs.create_dir("/storage2/").unwrap();

    let files_descriptors = dfs.readdir("/a/b/".to_string());
    assert_eq!(files_descriptors, vec![]);

    fs.create_file("/storage1/b/c").unwrap();
    fs.create_file("/storage2/c").unwrap();

    let files_descriptors = dfs.readdir("/a/b".to_string());
    assert_eq!(
        files_descriptors,
        vec![
            // Storage of the container claiming path `/a/` + `b/c` within the container gives full path `/a/b/c`
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    storage1,
                    PathBuf::from_str("/b/c").unwrap()
                )),
                absolute_path: PathBuf::from_str("/a/b/c").unwrap(),
            },
            // Storage of the container claiming path `/a/b` + `c` within the container also gives full path `/a/b/c`
            NodeDescriptor {
                storage: Some(NodeStorage::new(storage2, PathBuf::from_str("/c").unwrap())),
                absolute_path: PathBuf::from_str("/a/b/c").unwrap(),
            }
        ]
    );
}

#[rstest]
fn test_first_storage_unavailable(dfs_with_path_resolver_and_fs: DfsFixture) {
    let (mut dfs, path_resolver, fs) = dfs_with_path_resolver_and_fs;
    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    unsafe {
        (Rc::as_ptr(&path_resolver) as *mut MockPathResolver)
            .as_mut()
            .unwrap()
            .expect_resolve()
            .with(predicate::eq(Path::new("/")))
            .times(1)
            .returning({
                let storage1 = storage1;
                let storage2 = storage2.clone();
                move |_path| {
                    vec![PathWithStorages {
                        path_within_storage: "/".into(),
                        storages: Some(vec![storage1.clone(), storage2.clone()]),
                    }]
                }
            })
    };

    // don't create storage1 directory so readdir returned "No such file or directory" error
    // then dfs should choose storage2
    fs.create_dir("/storage2/").unwrap();
    fs.create_file("/storage2/a").unwrap();

    let files_descriptors = dfs.readdir("/".to_string());
    assert_eq!(
        files_descriptors,
        vec![NodeDescriptor {
            storage: Some(NodeStorage::new(storage2, PathBuf::from_str("/a").unwrap())),
            absolute_path: PathBuf::from_str("/a").unwrap(),
        },]
    );
}

#[rstest]
fn test_listing_virtual_node(dfs_with_path_resolver_and_fs: DfsFixture) {
    let (mut dfs, path_resolver, fs) = dfs_with_path_resolver_and_fs;
    // C1 storage
    let storage1 = new_mufs_storage("/storage_c1/");

    unsafe {
        (Rc::as_ptr(&path_resolver) as *mut MockPathResolver)
            .as_mut()
            .unwrap()
            .expect_resolve()
            .with(predicate::eq(Path::new("/a")))
            .times(1)
            .returning({
                let storage1 = storage1.clone();
                move |_path| {
                    vec![
                        PathWithStorages {
                            path_within_storage: "/".into(),
                            storages: Some(vec![storage1.clone()]),
                        },
                        // virtual storage (represented by a None value) represents containers
                        // that claim path containing the value of path_within_storage
                        // in this case container would claim path starting with /a/b/...
                        PathWithStorages {
                            path_within_storage: "/b".into(),
                            storages: None,
                        },
                    ]
                }
            })
    };

    fs.create_dir("/storage_c1/").unwrap();
    fs.create_file("/storage_c1/file_1").unwrap();
    fs.create_dir("/storage_c1/dir/").unwrap();
    fs.create_file("storage_c1/dir/file_in_nested_dir").unwrap(); // it should not be present in result

    let files_descriptors = dfs.readdir("/a".to_string());
    assert_eq!(
        files_descriptors,
        vec![
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    storage1.clone(),
                    PathBuf::from_str("/dir").unwrap()
                )),
                absolute_path: PathBuf::from_str("/a/dir").unwrap(),
            },
            NodeDescriptor {
                storage: Some(NodeStorage::new(
                    storage1,
                    PathBuf::from_str("/file_1").unwrap()
                )),
                absolute_path: PathBuf::from_str("/a/file_1").unwrap(),
            },
            NodeDescriptor {
                storage: None,
                absolute_path: PathBuf::from_str("/a/b").unwrap(),
            },
        ]
    );
}
