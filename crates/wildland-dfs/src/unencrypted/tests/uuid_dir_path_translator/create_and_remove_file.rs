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
use rsfs::{GenFS, Metadata};
use rstest::rstest;
use uuid::Uuid;
use wildland_corex::dfs::interface::{DfsFrontend, DfsFrontendError};
use wildland_corex::{MockPathResolver, ResolvedPath};

use crate::unencrypted::tests::{dfs_with_fs, new_mufs_storage};

#[rstest]
fn test_create_file_in_path_without_containers() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/file")))
        .times(1)
        .returning(move |_path| Ok(HashSet::new()));

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let err = dfs.create_file("/file".to_string()).unwrap_err();
    assert_eq!(err, DfsFrontendError::InvalidParent);
}

#[rstest]
fn test_create_file_in_path_without_parent() {
    let mut path_resolver = MockPathResolver::new();
    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/file")))
        .times(1)
        .returning({
            let storage = mufs_storage;
            move |_path| {
                Ok(HashSet::from([ResolvedPath::PathWithStorages {
                    path_within_storage: "/a/file".into(),
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage.clone()],
                }]))
            }
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let err = dfs.create_file("/a/file".to_string()).unwrap_err();
    assert_eq!(err, DfsFrontendError::InvalidParent);
}

#[rstest]
fn test_create_file_in_root_succeeds() {
    let mut path_resolver = MockPathResolver::new();
    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/file")))
        .times(1)
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
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    dfs.create_file("/file".to_string()).unwrap();
}

#[rstest]
fn test_create_file_conflicting_with_virtual_node() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/virtual_dir")))
        .times(1)
        .returning(move |_path| {
            Ok(HashSet::from([ResolvedPath::VirtualPath(PathBuf::from(
                "/virtual_dir/end_of_path",
            ))]))
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let err = dfs.create_file("/virtual_dir".to_string()).unwrap_err();
    assert_eq!(err, DfsFrontendError::PathAlreadyExists);
}

#[rstest]
fn test_create_file_when_path_resolver_returned_many_possible_paths() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b/file")))
        .times(1)
        .returning({
            let storage1 = storage1;
            let storage2 = storage2;
            move |_path| {
                Ok(HashSet::from([
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/b/file".into(), // returned by the container claiming path `/a/`
                        storages_id: Uuid::from_u128(1),
                        storages: vec![storage1.clone()],
                    },
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/file".into(), // returned by the container claiming path `/a/b/`
                        storages_id: Uuid::from_u128(2),
                        storages: vec![storage2.clone()],
                    },
                ]))
            }
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1").unwrap();
    fs.create_dir("/storage2").unwrap();

    dfs.create_file("/a/b/file".to_string()).unwrap();
    assert!(fs.metadata("/storage2/file").unwrap().is_file());
}

#[rstest]
fn test_create_file_in_ambiguous_path_should_fail() {
    let mut path_resolver = MockPathResolver::new();

    // each container has its own subfolder
    let storage1 = new_mufs_storage("/storage1/");
    let storage2 = new_mufs_storage("/storage2/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b/file")))
        .times(1)
        .returning({
            let storage1 = storage1;
            let storage2 = storage2;
            move |_path| {
                Ok(HashSet::from([
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/b/file".into(), // returned by the container claiming path `/a/`
                        storages_id: Uuid::from_u128(1),
                        storages: vec![storage1.clone()],
                    },
                    ResolvedPath::PathWithStorages {
                        path_within_storage: "/file".into(), // returned by the container claiming path `/a/b/`
                        storages_id: Uuid::from_u128(2),
                        storages: vec![storage2.clone()],
                    },
                ]))
            }
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/storage1").unwrap();
    fs.create_dir("/storage1/b").unwrap();
    fs.create_dir("/storage2").unwrap();

    let err = dfs.create_file("/a/b/file".to_string()).unwrap_err();
    assert_eq!(err, DfsFrontendError::ReadOnlyPath);
}

#[rstest]
fn test_remove_nonexistent_file() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/file")))
        .times(1)
        .returning(move |_path| Ok(HashSet::new()));

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let stat = dfs.remove_file("/a/file".to_string()).unwrap_err();
    assert_eq!(stat, DfsFrontendError::NoSuchPath)
}

#[rstest]
fn test_remove_nonexistent_file_when_path_resolver_returned_some_path() {
    let mut path_resolver = MockPathResolver::new();
    let storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/file")))
        .times(1)
        .returning(move |_path| {
            Ok(HashSet::from([ResolvedPath::PathWithStorages {
                path_within_storage: "/file".into(),
                storages_id: Uuid::from_u128(1),
                storages: vec![storage.clone()],
            }]))
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let stat = dfs.remove_file("/a/file".to_string()).unwrap_err();
    assert_eq!(stat, DfsFrontendError::NoSuchPath)
}

#[rstest]
fn test_remove_file_called_on_virtual_path() {
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

    let stat = dfs.remove_file("/virtual_dir".to_string()).unwrap_err();
    assert_eq!(stat, DfsFrontendError::ReadOnlyPath)
}

#[rstest]
fn test_remove_file_called_on_directory_path() {
    let mut path_resolver = MockPathResolver::new();
    let storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/dir")))
        .times(1)
        .returning(move |_path| {
            Ok(HashSet::from([ResolvedPath::PathWithStorages {
                path_within_storage: "/dir".into(),
                storages_id: Uuid::from_u128(1),
                storages: vec![storage.clone()],
            }]))
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_dir("/dir").unwrap();

    let stat = dfs.remove_file("/a/dir".to_string()).unwrap_err();
    assert_eq!(stat, DfsFrontendError::NotAFile)
}

#[rstest]
fn test_remove_file_should_succeed() {
    let mut path_resolver = MockPathResolver::new();
    let storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/file")))
        .times(1)
        .returning(move |_path| {
            Ok(HashSet::from([ResolvedPath::PathWithStorages {
                path_within_storage: "/file".into(),
                storages_id: Uuid::from_u128(1),
                storages: vec![storage.clone()],
            }]))
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_fs(path_resolver);

    fs.create_file("/file").unwrap();

    dfs.remove_file("/a/file".to_string()).unwrap();
}
