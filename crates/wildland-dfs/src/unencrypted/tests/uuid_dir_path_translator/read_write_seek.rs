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
use rsfs::GenFS;
use rstest::rstest;
use uuid::Uuid;
use wildland_corex::dfs::interface::DfsFrontend;
use wildland_corex::{MockPathResolver, ResolvedPath};

use crate::unencrypted::tests::{dfs_with_mu_fs, new_mufs_storage};

#[rstest]
fn test_read_empty_file() {
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
    let read_bytes = dfs.read(&file, 5).unwrap();
    assert_eq!(read_bytes, vec![0; 0]);
}

#[rstest]
fn test_write_and_seek_from_start_and_read() {
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

    dfs.write(&file, vec![1, 2, 3, 4, 5, 6]).unwrap();

    let read_bytes = dfs.read(&file, 5).unwrap();
    assert_eq!(read_bytes, vec![0; 0]);

    let pos = dfs.seek_from_start(&file, 1).unwrap();
    assert_eq!(pos, 1);

    let read_bytes = dfs.read(&file, 3).unwrap();
    assert_eq!(read_bytes, vec![2, 3, 4]);
}
