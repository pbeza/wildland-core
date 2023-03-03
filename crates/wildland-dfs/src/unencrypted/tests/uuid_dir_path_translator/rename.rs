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

use crate::unencrypted::tests::{dfs_with_mu_fs, new_mufs_storage};

#[rstest]
fn test_rename_of_nonexistent_path() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/file")))
        .times(1)
        .returning(move |_path| Ok(HashSet::new()));

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_mu_fs(path_resolver);

    let err = dfs.rename("/a/file".to_string(), "".into()).unwrap_err();
    assert_eq!(err, DfsFrontendError::NoSuchPath)
}

#[rstest]
fn test_rename_of_virtual() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/b")))
        .times(1)
        .returning(move |_path| {
            Ok(HashSet::from([ResolvedPath::VirtualPath(PathBuf::from(
                "/a/b/c",
            ))]))
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_mu_fs(path_resolver);

    let err = dfs.rename("/a/b".to_string(), "/a/c".into()).unwrap_err();
    assert_eq!(err, DfsFrontendError::ReadOnlyPath)
}

#[rstest]
fn test_successful_rename_of_file() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_file("/file").unwrap();

    dfs.rename("/a/file".to_string(), "/a/new_file".into())
        .unwrap();
    assert!(fs.metadata("/new_file").unwrap().is_file())
}

#[rstest]
fn test_rename_of_file_when_target_file_exists() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_file("/file").unwrap();
    fs.create_file("/new_file").unwrap();

    let err = dfs
        .rename("/a/file".to_string(), "/a/new_file".into())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::PathAlreadyExists);
}

#[rstest]
fn test_successful_rename_of_dir() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_dir("/dir").unwrap();

    dfs.rename("/a/dir".to_string(), "/a/new_dir".into())
        .unwrap();

    assert!(fs.metadata("/new_dir").unwrap().is_dir())
}

#[rstest]
fn test_target_path_with_no_parent() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_dir("/dir").unwrap();

    let err = dfs
        .rename("/a/dir".to_string(), "/a/does_not_exist/new_dir".into())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::NoSuchPath)
}

#[rstest]
fn test_target_path_is_subdir_of_old() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_dir("/dir").unwrap();

    let err = dfs
        .rename("/a/dir".to_string(), "/a/dir/new_dir".into())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::SourceIsParentOfTarget);
}

#[rstest]
fn test_target_path_is_dir_but_source_is_not() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_file("/file").unwrap();
    fs.create_dir("/dir").unwrap();

    let err = dfs
        .rename("/a/file".to_string(), "/a/dir".into())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::PathAlreadyExists);
}

#[rstest]
fn test_target_path_is_file_but_source_is_not() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_file("/file").unwrap();
    fs.create_dir("/dir").unwrap();

    let err = dfs
        .rename("/a/dir".to_string(), "/a/file".into())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::PathAlreadyExists);
}

#[rstest]
fn test_rename_directory_with_empty_dir_as_target() {
    let mut path_resolver = MockPathResolver::new();
    let storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/dir")))
        .times(1)
        .returning(move |_path| {
            Ok(HashSet::from([ResolvedPath::PathWithStorages {
                path_within_storage: "/dir".into(),
                storages_id: Uuid::from_u128(1),
                storages: vec![storage.clone()],
            }]))
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_dir("/dir").unwrap();
    fs.create_file("/dir/file").unwrap();
    fs.create_dir("/empty_dir").unwrap();

    let err = dfs
        .rename("/dir".to_string(), "/empty_dir".into())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::PathAlreadyExists);
}

#[rstest]
fn test_rename_directory_with_non_empty_dir_as_target() {
    let mut path_resolver = MockPathResolver::new();
    let storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/dir")))
        .times(1)
        .returning(move |_path| {
            Ok(HashSet::from([ResolvedPath::PathWithStorages {
                path_within_storage: "/dir".into(),
                storages_id: Uuid::from_u128(1),
                storages: vec![storage.clone()],
            }]))
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_dir("/dir").unwrap();
    fs.create_file("/dir/file").unwrap();
    fs.create_dir("/non_empty_dir").unwrap();
    fs.create_file("/non_empty_dir/file").unwrap();

    let err = dfs
        .rename("/dir".to_string(), "/non_empty_dir".into())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::PathAlreadyExists);
}

#[rstest]
fn test_rename_between_containers() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_dir("/dir").unwrap();

    let err = dfs
        .rename("/a/dir".to_string(), "/b/dir".into())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::MoveBetweenContainers);
}

#[rstest]
fn test_rename_conflicting_files() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_dir("/storage1").unwrap();
    fs.create_dir("/storage1/b").unwrap();
    fs.create_file("/storage1/b/file").unwrap();
    fs.create_dir("/storage2").unwrap();
    fs.create_file("/storage2/file").unwrap();

    let err = dfs
        .rename("/a/b/file".to_string(), "/a/b/new_file".into())
        .unwrap_err();
    assert_eq!(err, DfsFrontendError::ReadOnlyPath);
}
