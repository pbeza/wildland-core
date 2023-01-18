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
use wildland_corex::dfs::interface::DfsFrontend;
use wildland_corex::{MockPathResolver, ResolvedPath};

use crate::unencrypted::tests::{dfs_with_fs, new_mufs_storage};

#[rstest]
fn test_listing_files_from_root_of_one_container() {
    let mut path_resolver = MockPathResolver::new();
    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b/")))
        .times(2)
        .returning({
            move |_path| {
                vec![ResolvedPath::PathWithStorages {
                    path_within_storage: "/".into(),
                    storages_id: Uuid::from_u128(1),
                    storages: vec![mufs_storage.clone()],
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

    let files = dfs.readdir("/a/b/".to_string());
    assert_eq!(files, Vec::<String>::new());

    fs.create_file("/file_in_root").unwrap();
    let files = dfs.readdir("/a/b/".to_string());
    assert_eq!(files, vec!["/a/b/file_in_root".to_string(),]);
}

#[rstest]
fn test_listing_files_from_nested_dir_of_one_container() {
    let mut path_resolver = MockPathResolver::new();

    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b/dir")))
        .times(2)
        .returning(move |_path| {
            vec![ResolvedPath::PathWithStorages {
                path_within_storage: "/dir".into(), // claim /a/b
                storages_id: Uuid::from_u128(1),
                storages: vec![mufs_storage.clone()],
            }]
        });

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::always())
        .times(2)
        .returning(move |_path| false);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    let files = dfs.readdir("/a/b/dir".to_string());
    assert_eq!(files, Vec::<String>::new());

    fs.create_dir("/dir/").unwrap();
    fs.create_file("/dir/nested_file_1").unwrap();
    fs.create_file("/dir/nested_file_2").unwrap();

    let files = dfs.readdir("/a/b/dir".to_string());
    assert_eq!(
        files,
        vec![
            "/a/b/dir/nested_file_1".to_string(),
            "/a/b/dir/nested_file_2".to_string(),
        ]
    );
}

#[rstest]
fn test_listing_dirs_from_one_container() {
    let mut path_resolver = MockPathResolver::new();

    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/")))
        .times(2)
        .returning(move |_path| {
            vec![ResolvedPath::PathWithStorages {
                path_within_storage: "/".into(), // claim /
                storages_id: Uuid::from_u128(1),
                storages: vec![mufs_storage.clone()],
            }]
        });

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::always())
        .times(2)
        .returning(move |_path| false);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    let files = dfs.readdir("/".to_string());
    assert_eq!(files, Vec::<String>::new());

    fs.create_dir("/dir_a").unwrap();
    fs.create_dir("/dir_b").unwrap();

    let files = dfs.readdir("/".to_string());
    assert_eq!(files, vec!["/dir_a".to_string(), "/dir_b".to_string(),]);
}

#[rstest]
fn test_listing_files_and_dirs_from_two_containers() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::always())
        .times(3)
        .returning(move |_path| false);

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b/c/dir")))
        .times(2)
        .returning(move |_path| {
            vec![
                ResolvedPath::PathWithStorages {
                    path_within_storage: "/dir".into(), // returned by a container claiming path `/a/b/c/`
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage1.clone()],
                },
                ResolvedPath::PathWithStorages {
                    path_within_storage: "/c/dir".into(), // returned by a container claiming path `/a/b/`
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage2.clone()],
                },
            ]
        });

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/dir/").unwrap();
    fs.create_dir("/storage2/").unwrap();
    fs.create_dir("/storage2/c/").unwrap();
    fs.create_dir("/storage2/c/dir/").unwrap();

    let files = dfs.readdir("/a/b/c/dir".to_string());
    assert_eq!(files, Vec::<String>::new());

    fs.create_file("/storage1/dir/file_from_container_1")
        .unwrap();
    fs.create_dir("/storage2/c/dir/next_dir").unwrap();
    fs.create_file("/storage2/c/dir/file_from_container_2")
        .unwrap();

    let files = dfs.readdir("/a/b/c/dir".to_string());
    assert_eq!(
        files,
        vec![
            "/a/b/c/dir/file_from_container_1".to_string(),
            "/a/b/c/dir/file_from_container_2".to_string(),
            "/a/b/c/dir/next_dir".to_string(),
        ]
    );
}

#[rstest]
fn test_getting_one_file_from_container_with_multiple_storages() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a")))
        .times(2)
        .returning(move |_path| {
            vec![ResolvedPath::PathWithStorages {
                path_within_storage: "/a".into(), // returned by a container claiming path `/`
                storages_id: Uuid::from_u128(1),
                storages: vec![storage1.clone(), storage2.clone()],
            }]
        });

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::always())
        .times(1)
        .returning(move |_path| false);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/a").unwrap();
    fs.create_dir("/storage2/").unwrap();
    fs.create_dir("/storage2/a").unwrap();

    let files = dfs.readdir("/a".to_string());
    assert_eq!(files, Vec::<String>::new());

    fs.create_file("/storage1/a/b").unwrap();
    fs.create_file("/storage2/a/b").unwrap();

    let files = dfs.readdir("/a".to_string());
    assert_eq!(files, vec!["/a/b".to_string(),]);
}

/// Full Path is a concatenation of a path claimed by a container with a path of the file inside the container.
#[rstest]
fn test_more_than_one_file_claim_the_same_full_path() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b/")))
        .times(2)
        .returning(move |_path| {
            vec![
                ResolvedPath::PathWithStorages {
                    path_within_storage: "/b/".into(), // returned by the container claiming path `/a/`
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage1.clone()],
                },
                ResolvedPath::PathWithStorages {
                    path_within_storage: "/".into(), // returned by the container claiming path `/a/b/`
                    storages_id: Uuid::from_u128(2),
                    storages: vec![storage2.clone()],
                },
            ]
        });

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/b").unwrap();
    fs.create_dir("/storage2/").unwrap();

    let files = dfs.readdir("/a/b/".to_string());
    assert_eq!(files, Vec::<String>::new());

    fs.create_file("/storage1/b/c").unwrap();
    fs.create_file("/storage2/c").unwrap();

    let files = dfs.readdir("/a/b".to_string());
    assert_eq!(
        files,
        vec![
            // Storage of the container claiming path `/a/` + `b/c` within the container gives full path `/a/b/c`
            // Storage of the container claiming path `/a/b` + `c` within the container also gives full path `/a/b/c`
            // The following dir contains both files with container uuids as names
            "/a/b/c/".to_string(),
        ]
    );
}

#[rstest]
fn test_first_storage_unavailable() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/")))
        .times(1)
        .returning(move |_path| {
            vec![ResolvedPath::PathWithStorages {
                path_within_storage: "/".into(), // claim /
                storages_id: Uuid::from_u128(1),
                storages: vec![storage1.clone(), storage2.clone()],
            }]
        });

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::always())
        .times(1)
        .returning(move |_path| false);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    // don't create storage1 directory so readdir returned "No such file or directory" error
    // then dfs should choose storage2
    fs.create_dir("/storage2/").unwrap();
    fs.create_file("/storage2/a").unwrap();

    let files = dfs.readdir("/".to_string());
    assert_eq!(files, vec!["/a".to_string(),]);
}

#[rstest]
fn test_listing_virtual_node() {
    let mut path_resolver = MockPathResolver::new();

    // C1 storage
    let storage1 = new_mufs_storage("/storage_c1/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a")))
        .times(1)
        .returning(move |_path| {
            vec![
                ResolvedPath::PathWithStorages {
                    path_within_storage: "/".into(), // claim /a
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage1.clone()],
                },
                ResolvedPath::VirtualPath("/a".into()), // returned if there is for example container claiming /a/b
            ]
        });

    path_resolver
        .expect_list_virtual_nodes_in()
        .with(predicate::eq(Path::new("/a")))
        .times(1)
        .returning(move |_path| vec!["b".to_string()]); // returned by containers claiming path /a/b/*

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::eq(Path::new("/a/dir")))
        .times(1)
        .returning(move |_path| false);
    path_resolver
        .expect_is_virtual_node()
        .with(predicate::eq(Path::new("/a/file_1")))
        .times(1)
        .returning(move |_path| false);
    path_resolver
        .expect_is_virtual_node()
        .with(predicate::eq(Path::new("/a/b")))
        .times(1)
        .returning(move |_path| true);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage_c1/").unwrap();
    fs.create_file("/storage_c1/file_1").unwrap();
    fs.create_dir("/storage_c1/dir/").unwrap();
    fs.create_file("storage_c1/dir/file_in_nested_dir").unwrap(); // it should not be present in result

    let files = dfs.readdir("/a".to_string());
    assert_eq!(
        files,
        vec![
            "/a/dir".to_string(),
            "/a/file_1".to_string(),
            "/a/b/".to_string(),
        ]
    );
}

#[rstest]
fn test_file_colliding_with_virtual_node() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/")))
        .times(1)
        .returning({
            let storage1 = storage1.clone();
            move |_path| {
                vec![
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/".into(), // returned by the container claiming path `/a/`
                        storages_id: Uuid::from_u128(1),
                        storages: vec![storage1.clone()],
                    },
                    ResolvedPath::VirtualPath(PathBuf::from("/a")), // returned if there is for example container claiming /a/b
                ]
            }
        });

    path_resolver
        .expect_list_virtual_nodes_in()
        .with(predicate::eq(Path::new("/a")))
        .times(1)
        .returning(move |_path| vec!["b".to_string()]); // returned by containers claiming path /a/b/*

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b")))
        .times(1)
        .returning(move |_path| {
            vec![
                ResolvedPath::PathWithStorages {
                    path_within_storage: "/b".into(), // returned by the container claiming path `/a/`
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage1.clone()],
                },
                ResolvedPath::VirtualPath(PathBuf::from("/a/b/")), // returned if there is for example container claiming /a/b/c
            ]
        });

    path_resolver
        .expect_list_virtual_nodes_in()
        .with(predicate::eq(Path::new("/a/b/")))
        .times(1)
        .returning(move |_path| vec![]);

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::eq(Path::new("/a/b/")))
        .times(1)
        .returning(move |_path| true);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_file("/storage1/b").unwrap();

    let files = dfs.readdir("/a/".to_string());
    // We expect only /a/b/ path represented by a directory.
    // File /b in container claiming path /a collides with virtual node /a/b so it should be exposed as
    // /a/b/00000000-0000-0000-0000-000000000001
    assert_eq!(files, vec!["/a/b/".to_string(),]);

    let files = dfs.readdir("/a/b/".to_string());
    assert_eq!(
        files,
        vec!["/a/b/00000000-0000-0000-0000-000000000001".to_string(),]
    );
}

#[rstest]
fn test_dir_colliding_with_virtual_node() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/")))
        .times(1)
        .returning({
            let storage1 = storage1.clone();
            move |_path| {
                vec![
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/".into(), // returned by the container claiming path `/a/`
                        storages_id: Uuid::from_u128(1),
                        storages: vec![storage1.clone()],
                    },
                    ResolvedPath::VirtualPath(PathBuf::from("/a")), // returned if there is for example container claiming /a/b
                ]
            }
        });

    path_resolver
        .expect_list_virtual_nodes_in()
        .with(predicate::eq(Path::new("/a")))
        .times(1)
        .returning(move |_path| vec!["b".to_string()]); // returned by containers claiming path /a/b/*

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b")))
        .times(1)
        .returning(move |_path| {
            vec![ResolvedPath::PathWithStorages {
                path_within_storage: "/b/".into(), // returned by the container claiming path `/a/`
                storages_id: Uuid::from_u128(1),
                storages: vec![storage1.clone()],
            }]
        });

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::eq(Path::new("/a/b/c")))
        .times(1)
        .returning(|_path| false);

    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/b").unwrap();
    fs.create_file("/storage1/b/c").unwrap();

    let files = dfs.readdir("/a/".to_string());
    // We expect only /a/b/ path represented by a directory.
    // Directory /b in container claiming path /a collides with virtual node /a/b so they are merged
    assert_eq!(files, vec!["/a/b/".to_string(),]);

    let files = dfs.readdir("/a/b/".to_string());
    assert_eq!(files, vec!["/a/b/c".to_string(),]);
}

#[rstest]
fn test_readdir_on_file() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/")))
        .times(2)
        .returning({
            let storage1 = storage1;
            move |_path| {
                vec![ResolvedPath::PathWithStorages {
                    path_within_storage: "/a".into(), // returned by the container claiming path `/`
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage1.clone()],
                }]
            }
        });

    path_resolver
        .expect_is_virtual_node()
        .with(predicate::eq(Path::new("/a")))
        .times(2)
        .returning(|_path| false);
    let path_resolver = Rc::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_file("/storage1/a").unwrap();

    let files = dfs.readdir("/a/".to_string());
    assert_eq!(files, Vec::<String>::new());
    let files = dfs.readdir("/a".to_string());
    assert_eq!(files, Vec::<String>::new());
}
