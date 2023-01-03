use crate::{
    storage_backend::StorageBackend,
    unencrypted::{StorageBackendFactory, UnencryptedDfs},
};
use mockall::predicate;
use pretty_assertions::assert_eq;
use rsfs::{mem::FS, DirEntry, GenFS};
use rstest::{fixture, rstest};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};
use wildland_corex::{
    dfs::interface::{DfsFrontend, NodeDescriptor},
    MockPathResolver, PathWithStorages, Storage,
};

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
                    vec![PathWithStorages {
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
            storage: mufs_storage,
            path: PathBuf::from_str("/file_in_root").unwrap()
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
                storage: mufs_storage,
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
                    vec![PathWithStorages {
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
                storage: mufs_storage,
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
                        PathWithStorages {
                            path: "/dir".into(), // returned by a container claiming path `/a/b/c/`
                            storages: vec![storage1.clone()],
                        },
                        PathWithStorages {
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
                storage: storage1,
                path: PathBuf::from_str("/dir/file_from_container_1").unwrap()
            },
            NodeDescriptor {
                storage: storage2.clone(),
                path: PathBuf::from_str("/c/dir/file_from_container_2").unwrap()
            },
            NodeDescriptor {
                storage: storage2,
                path: PathBuf::from_str("/c/dir/next_dir").unwrap()
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
                        path: "/a".into(), // returned by a container claiming path `/a/`
                        storages: vec![storage1.clone(), storage2.clone()],
                    }]
                }
            })
    };

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/a").unwrap();
    fs.create_dir("/storage2/").unwrap();
    fs.create_dir("/storage2/a").unwrap();

    let files_descriptors = dfs.readdir("/a");
    assert_eq!(files_descriptors, vec![]);

    fs.create_file("/storage1/a/b").unwrap();
    fs.create_file("/storage2/a/b").unwrap();

    let files_descriptors = dfs.readdir("/a");
    assert_eq!(
        files_descriptors,
        vec![NodeDescriptor {
            storage: storage1,
            path: PathBuf::from_str("/a/b").unwrap(),
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
                            path: "/b/".into(), // returned by the container claiming path `/a/`
                            storages: vec![storage1.clone()],
                        },
                        PathWithStorages {
                            path: "/".into(), // returned by the container claiming path `/a/b/`
                            storages: vec![storage2.clone()],
                        },
                    ]
                }
            })
    };

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/b").unwrap();
    fs.create_dir("/storage2/").unwrap();

    let files_descriptors = dfs.readdir("/a/b/");
    assert_eq!(files_descriptors, vec![]);

    fs.create_file("/storage1/b/c").unwrap();
    fs.create_file("/storage2/c").unwrap();

    let files_descriptors = dfs.readdir("/a/b");
    assert_eq!(
        files_descriptors,
        vec![
            // Storage of the container claiming path `/a/` + `b/c` within the container gives full path `/a/b/c`
            NodeDescriptor {
                storage: storage1,
                path: PathBuf::from_str("/b/c").unwrap(),
            },
            // Storage of the container claiming path `/a/b` + `c` within the container also gives full path `/a/b/c`
            NodeDescriptor {
                storage: storage2,
                path: PathBuf::from_str("/c").unwrap(),
            }
        ]
    );
}
