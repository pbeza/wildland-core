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
use std::path::Path;

use mockall::predicate;
use pretty_assertions::assert_eq;
use rsfs::GenFS;
use rstest::rstest;
use uuid::Uuid;
use wildland_corex::dfs::interface::{DfsFrontend, DfsFrontendError};
use wildland_corex::{MockPathResolver, ResolvedPath};

use crate::unencrypted::tests::{dfs_with_mu_fs, new_mufs_storage};

#[rstest]
fn test_open_of_nonexistent_path() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/file")))
        .times(1)
        .returning(move |_path| Ok(HashSet::new()));

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_mu_fs(path_resolver);

    let stat = dfs.open("/a/file".to_string()).unwrap_err();
    assert_eq!(stat, DfsFrontendError::NoSuchPath)
}

#[rstest]
fn test_open_of_file_in_container_root() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_file("/file").unwrap();

    let _file = dfs.open("/file".into()).unwrap();
}

#[rstest]
fn test_open_the_same_file_twice() {
    let mut path_resolver = MockPathResolver::new();
    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/file")))
        .times(2)
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_file("/file").unwrap();

    let _file = dfs.open("/file".into()).unwrap();
    let _file = dfs.open("/file".into()).unwrap();
}

#[rstest]
fn test_open_and_close_file() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_file("/file").unwrap();

    let file = dfs.open("/file".into()).unwrap();
    dfs.close(&file).unwrap();
}

#[rstest]
fn test_open_and_close_the_same_file_twice() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_file("/file").unwrap();

    let file = dfs.open("/file".into()).unwrap();
    dfs.close(&file).unwrap();
    assert_eq!(
        DfsFrontendError::FileAlreadyClosed,
        dfs.close(&file).unwrap_err()
    );
}

#[rstest]
fn test_open_and_close_conflicting_files() {
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
    let (mut dfs, fs) = dfs_with_mu_fs(path_resolver);

    fs.create_dir("/storage1/").unwrap();
    fs.create_dir("/storage1/b").unwrap();
    fs.create_file("/storage1/b/file_or_dir").unwrap();
    fs.create_dir("/storage2/").unwrap();
    fs.create_dir("/storage2/file_or_dir").unwrap();

    assert_eq!(
        DfsFrontendError::NotAFile,
        dfs.open("/a/b/file_or_dir".into()).unwrap_err()
    );

    let file = dfs
        .open("/a/b/file_or_dir/00000000-0000-0000-0000-000000000001".into())
        .unwrap();
    dfs.close(&file).unwrap();

    assert_eq!(
        DfsFrontendError::NotAFile,
        dfs.open("/a/b/file_or_dir/00000000-0000-0000-0000-000000000002".into())
            .unwrap_err()
    );
}
