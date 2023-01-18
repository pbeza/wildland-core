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

use std::path::{Path, PathBuf};
use std::rc::Rc;

use mockall::predicate;
use pretty_assertions::assert_eq;
use rsfs::GenFS;
use rstest::rstest;
use uuid::Uuid;
use wildland_corex::dfs::interface::{DfsFrontend, NodeType, Stat};
use wildland_corex::{MockPathResolver, ResolvedPath};

use crate::unencrypted::tests::{dfs_with_fs, get_unix_time_of_file, new_mufs_storage, MufsAttrs};

#[rstest]
fn test_getattr_of_nonexistent_path() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/file")))
        .times(1)
        .returning(move |_path| vec![]);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let stat = dfs.getattr("/a/file".to_string());
    assert_eq!(stat, None)
}

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

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::always())
        .times(1)
        .returning(move |_path| false);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_file("/file").unwrap();
    let MufsAttrs {
        access_time,
        modification_time,
        change_time,
        size,
    } = get_unix_time_of_file("/file", &fs);

    let stat = dfs.getattr("/file".to_string());
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::File,
            access_time,
            modification_time,
            change_time,
            size
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

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::always())
        .times(1)
        .returning(move |_path| false);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/dir").unwrap();
    let MufsAttrs {
        access_time,
        modification_time,
        change_time,
        size,
    } = get_unix_time_of_file("/dir", &fs);

    let stat = dfs.getattr("/dir".to_string());
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::Dir,
            access_time,
            modification_time,
            change_time,
            size
        })
    )
}

#[rstest]
fn test_getattr_of_virtual_dir() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/virtual_dir"))) // e.g. container claiming path /virtual_dir/something
        .times(1)
        .returning(move |_path| vec![ResolvedPath::VirtualPath(PathBuf::from("/virtual_dir"))]);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let stat = dfs.getattr("/virtual_dir".to_string());
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::Dir,
            access_time: None,
            modification_time: None,
            change_time: None,
            size: 0
        })
    )
}

#[rstest]
fn test_getattr_of_conflicting_path_using_container_uuid() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b/file_or_dir")))
        .times(3)
        .returning({
            let storage1 = storage1;
            let storage2 = storage2;
            move |_path| {
                vec![
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/b/file_or_dir".into(), // returned by the container claiming path `/a/`
                        storages_id: Uuid::from_u128(1),
                        storages: vec![storage1.clone()],
                    },
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/file_or_dir".into(), // returned by the container claiming path `/a/b/`
                        storages_id: Uuid::from_u128(2),
                        storages: vec![storage2.clone()],
                    },
                ]
            }
        });

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/b").unwrap();
    fs.create_file("/storage1/b/file_or_dir").unwrap();
    fs.create_dir("/storage2/").unwrap();
    fs.create_dir("/storage2/file_or_dir").unwrap();

    let stat = dfs.getattr("/a/b/file_or_dir/00000000-0000-0000-0000-000000000001".to_string());
    let MufsAttrs {
        access_time,
        modification_time,
        change_time,
        size,
    } = get_unix_time_of_file("/storage1/b/file_or_dir", &fs);
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::File,
            access_time,
            modification_time,
            change_time,
            size
        })
    );

    // getattr of aggregating dir
    let stat = dfs.getattr("/a/b/file_or_dir".to_string());
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::Dir,
            access_time: None,
            modification_time: None,
            change_time: None,
            size: 0
        })
    );

    // Directory /a/b/file_or_dir from storage2 can be accessed by appending path with its uuid.
    // It is hard to tell whether it is a bug or a feature, bit if it is a bug it is not trivial to fix.
    let stat = dfs.getattr("/a/b/file_or_dir/00000000-0000-0000-0000-000000000002".to_string());
    let MufsAttrs {
        access_time,
        modification_time,
        change_time,
        size,
    } = get_unix_time_of_file("/storage2/file_or_dir", &fs);
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::Dir,
            access_time,
            modification_time,
            change_time,
            size
        })
    );
}

#[rstest]
fn test_virtual_path_colliding_with_file() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b")))
        .times(2)
        .returning({
            move |_path| {
                vec![
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/b".into(), // returned by the container claiming path `/a/`
                        storages_id: Uuid::from_u128(1),
                        storages: vec![storage1.clone()],
                    },
                    ResolvedPath::VirtualPath(PathBuf::from("/a/b")), // returned by containers claiming path `/a/b/*`
                ]
            }
        });

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_file("/storage1/b").unwrap();

    // /a/b should be a dir
    let stat = dfs.getattr("/a/b/".to_string());
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::Dir,
            access_time: None,
            modification_time: None,
            change_time: None,
            size: 0
        })
    );

    // file /b from container claiming /a should be represented with appended container uuid to avoid collision
    let stat = dfs.getattr("/a/b/00000000-0000-0000-0000-000000000001".to_string());
    let MufsAttrs {
        access_time,
        modification_time,
        change_time,
        size,
    } = get_unix_time_of_file("/storage1/b", &fs);
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::File,
            access_time,
            modification_time,
            change_time,
            size
        })
    )
}

#[rstest]
fn test_virtual_path_colliding_with_dir() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b")))
        .times(2)
        .returning({
            move |_path| {
                vec![
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/b".into(), // returned by the container claiming path `/a/`
                        storages_id: Uuid::from_u128(1),
                        storages: vec![storage1.clone()],
                    },
                    ResolvedPath::VirtualPath(PathBuf::from("/a/b")), // returned by containers claiming path `/a/b/*`
                ]
            }
        });

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/b").unwrap();

    // /a/b should be a dir
    let stat = dfs.getattr("/a/b/".to_string());
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::Dir,
            access_time: None,
            modification_time: None,
            change_time: None,
            size: 0
        })
    );

    // file /b from container claiming /a should be represented with appended container uuid to avoid collision
    let stat = dfs.getattr("/a/b/00000000-0000-0000-0000-000000000001".to_string());
    let MufsAttrs {
        access_time,
        modification_time,
        change_time,
        size,
    } = get_unix_time_of_file("/storage1/b", &fs);
    assert_eq!(
        stat,
        Some(Stat {
            node_type: NodeType::Dir,
            access_time,
            modification_time,
            change_time,
            size
        })
    )
}
