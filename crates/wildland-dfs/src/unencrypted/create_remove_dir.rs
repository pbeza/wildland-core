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

use std::path::PathBuf;
use std::str::FromStr;

use wildland_corex::dfs::interface::DfsFrontendError;

use super::utils::{execute_container_operation, get_related_nodes};
use super::{NodeDescriptor, UnencryptedDfs};
use crate::storage_backends::{CreateDirResponse, RemoveDirResponse};

pub fn create_dir(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
) -> Result<(), DfsFrontendError> {
    let requested_path = PathBuf::from_str(&requested_path).unwrap();
    let mut nodes = get_related_nodes(dfs, &requested_path)?;

    match nodes.len() {
        0 => Err(DfsFrontendError::ParentDoesNotExist),
        1 => match nodes.pop().unwrap() {
            NodeDescriptor::Physical { storages, .. } => {
                execute_container_operation(dfs, &storages, |backend, path| {
                    backend.create_dir(path)
                })
                .and_then(|resp| match resp {
                    CreateDirResponse::Created => Ok(()),
                    CreateDirResponse::ParentDoesNotExist => {
                        Err(DfsFrontendError::ParentDoesNotExist)
                    }
                    CreateDirResponse::PathAlreadyExists => {
                        Err(DfsFrontendError::PathAlreadyExists)
                    }
                })
            }
            NodeDescriptor::Virtual { .. } => Err(DfsFrontendError::PathAlreadyExists),
        },
        _ => Err(DfsFrontendError::ReadOnlyPath), // We treat folders that are merge of content from many containers are read-only
    }
}

pub fn remove_dir(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
) -> Result<(), DfsFrontendError> {
    let requested_path = PathBuf::from_str(&requested_path).unwrap();
    let mut nodes = get_related_nodes(dfs, &requested_path)?;

    match nodes.len() {
        0 => Err(DfsFrontendError::NoSuchPath),
        1 => match nodes.pop().unwrap() {
            NodeDescriptor::Physical { storages, .. } => {
                execute_container_operation(dfs, &storages, |backend, path| {
                    backend.remove_dir(path)
                })
                .and_then(|resp| match resp {
                    RemoveDirResponse::Removed => Ok(()),
                    RemoveDirResponse::NotFound => Err(DfsFrontendError::NoSuchPath),
                    RemoveDirResponse::NotADirectory => Err(DfsFrontendError::NotADirectory),
                    RemoveDirResponse::DirNotEmpty => Err(DfsFrontendError::DirNotEmpty),
                })
            }
            NodeDescriptor::Virtual { .. } => Err(DfsFrontendError::ReadOnlyPath),
        },
        _ => Err(DfsFrontendError::ReadOnlyPath), // We treat folders that are merge of content from many containers are read-only
    }
}
