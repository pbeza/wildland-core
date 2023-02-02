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
use wildland_corex::dfs::interface::{DfsFrontend, DfsFrontendError};
use wildland_corex::{MockPathResolver, ResolvedPath};

use crate::unencrypted::tests::{dfs_with_fs, new_mufs_storage};

#[rstest]
fn test_create_dir_in_path_without_containers() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/dir")))
        .times(1)
        .returning(move |_path| Ok(HashSet::new()));

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let stat = dfs.create_dir("/dir".to_string()).unwrap_err();
    assert_eq!(stat, DfsFrontendError::ParentDoesNotExist);
}

#[rstest]
fn test_create_dir_in_path_without_parent() {
    let mut path_resolver = MockPathResolver::new();
    let mufs_storage = new_mufs_storage("/");

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/a/dir")))
        .times(1)
        .returning({
            let storage = mufs_storage;
            move |_path| {
                Ok(HashSet::from([ResolvedPath::PathWithStorages {
                    path_within_storage: "/a/dir".into(),
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage.clone()],
                }]))
            }
        });

    let path_resolver = Box::new(path_resolver);
    let (mut dfs, _fs) = dfs_with_fs(path_resolver);

    let stat = dfs.create_dir("/a/dir".to_string()).unwrap_err();
    assert_eq!(stat, DfsFrontendError::ParentDoesNotExist);
}
