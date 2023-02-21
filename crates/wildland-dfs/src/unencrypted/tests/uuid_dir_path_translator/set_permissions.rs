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

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use mockall::predicate;
use rsfs::GenFS;
use rstest::rstest;
use uuid::Uuid;
use wildland_corex::dfs::interface::{DfsFrontend, DfsFrontendError, WlPermissions};
use wildland_corex::{MockPathResolver, ResolvedPath};

use crate::unencrypted::tests::{dfs_with_fs, new_mufs_storage};

#[rstest]
fn test_set_permissions_of_nonexistent_path() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/file")))
        .times(1)
        .returning(move |_path| Ok(HashSet::new()));

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let stat = dfs.metadata("/a/file".to_string()).unwrap_err();
    assert_eq!(stat, DfsFrontendError::NoSuchPath)
}

#[rstest]
fn test_set_permissions_of_file_in_container_root() {
    let mut path_resolver = MockPathResolver::new();
    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/file")))
        .times(4)
        .returning({
            let storage = mufs_storage;
            move |_path| {
                Ok(HashSet::from([ResolvedPath::PathWithStorages {
                    path_within_storage: "/file".into(),
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage.clone()],
                }]))
            }
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_file("/file").unwrap();

    dfs.set_permissions("/file".to_string(), WlPermissions::readonly())
        .unwrap();
    assert!(dfs
        .metadata("/file".into())
        .unwrap()
        .permissions
        .is_readonly());
    dfs.set_permissions("/file".to_string(), WlPermissions::read_write())
        .unwrap();
    assert!(!dfs
        .metadata("/file".into())
        .unwrap()
        .permissions
        .is_readonly());
}

#[rstest]
fn test_set_permissions_of_dir_in_container_root() {
    let mut path_resolver = MockPathResolver::new();
    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/dir")))
        .times(4)
        .returning({
            let storage = mufs_storage;
            move |_path| {
                Ok(HashSet::from([ResolvedPath::PathWithStorages {
                    path_within_storage: "/dir".into(),
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage.clone()],
                }]))
            }
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/dir").unwrap();

    dfs.set_permissions("/dir".to_string(), WlPermissions::readonly())
        .unwrap();
    assert!(dfs
        .metadata("/dir".into())
        .unwrap()
        .permissions
        .is_readonly());
    dfs.set_permissions("/dir".to_string(), WlPermissions::read_write())
        .unwrap();
    assert!(!dfs
        .metadata("/dir".into())
        .unwrap()
        .permissions
        .is_readonly());
}

#[rstest]
fn test_set_permissions_of_virtual_dir() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/virtual_dir"))) // e.g. container claiming path /virtual_dir/something
        .times(1)
        .returning(move |_path| {
            Ok(HashSet::from([ResolvedPath::VirtualPath(PathBuf::from(
                "/virtual_dir",
            ))]))
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let err = dfs
        .set_permissions("/virtual_dir".to_string(), WlPermissions::read_write())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::ReadOnlyPath);
}

#[rstest]
fn test_set_permissions_of_conflicting_paths() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b/file_or_dir")))
        .times(1)
        .returning({
            let storage1 = storage1;
            let storage2 = storage2;
            move |_path| {
                Ok(HashSet::from([
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
                ]))
            }
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/b").unwrap();
    fs.create_file("/storage1/b/file_or_dir").unwrap();
    fs.create_dir("/storage2/").unwrap();
    fs.create_dir("/storage2/file_or_dir").unwrap();

    let err = dfs
        .set_permissions("/a/b/file_or_dir".to_string(), WlPermissions::read_write())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::ReadOnlyPath);
}

#[rstest]
fn test_virtual_path_colliding_with_file() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b")))
        .times(1)
        .returning({
            move |_path| {
                Ok(HashSet::from([
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/b".into(), // returned by the container claiming path `/a/`
                        storages_id: Uuid::from_u128(1),
                        storages: vec![storage1.clone()],
                    },
                    ResolvedPath::VirtualPath(PathBuf::from("/a/b")), // returned by containers claiming path `/a/b/*`
                ]))
            }
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_file("/storage1/b").unwrap();

    let err = dfs
        .set_permissions("/a/b".to_string(), WlPermissions::read_write())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::ReadOnlyPath);
}
