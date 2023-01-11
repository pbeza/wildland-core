use std::path::{Path, PathBuf};
use std::rc::Rc;

use mockall::predicate;
use rsfs::GenFS;
use rstest::rstest;
use uuid::Uuid;
use wildland_corex::dfs::interface::{DfsFrontend, NodeType, Stat};
use wildland_corex::{MockPathResolver, ResolvedPath};

use super::{dfs_with_fs, new_mufs_storage};

#[rstest]
fn test_getattr_of_file_in_container_root() {
    let mut path_resolver = MockPathResolver::new();
    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/file")))
        .times(1)
        .returning({
            let storage = mufs_storage;
            move |_path| {
                vec![ResolvedPath::PathWithStorages {
                    path_within_storage: "/file".into(),
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage.clone()],
                }]
            }
        });

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_file("/file").unwrap();

    let stat = dfs.getattr("/file".to_string());
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::File
        })
    )
}

#[rstest]
fn test_getattr_of_dir_in_container_root() {
    let mut path_resolver = MockPathResolver::new();
    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/dir")))
        .times(1)
        .returning({
            let storage = mufs_storage;
            move |_path| {
                vec![ResolvedPath::PathWithStorages {
                    path_within_storage: "/dir".into(),
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage.clone()],
                }]
            }
        });

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/dir").unwrap();

    let stat = dfs.getattr("/dir".to_string());
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::Dir
        })
    )
}

#[rstest]
fn test_getattr_of_virtual_dir() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/virtual_dir")))
        .times(1)
        .returning(move |_path| vec![ResolvedPath::VirtualPath(PathBuf::from("/"))]);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let stat = dfs.getattr("/virtual_dir".to_string());
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::Dir
        })
    )
}
