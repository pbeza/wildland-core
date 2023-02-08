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

use super::utils::{execute_container_operation, fetch_data_from_containers, get_related_nodes};
use super::{NodeDescriptor, UnencryptedDfs};
use crate::storage_backends::{CreateDirResponse, RemoveDirResponse};

pub fn create_dir(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
) -> Result<(), DfsFrontendError> {
    let requested_path = PathBuf::from_str(&requested_path).unwrap();
    let mut nodes = get_related_nodes(dfs, &requested_path)?;

    let create_dir_in_container = |dfs: &mut UnencryptedDfs, node: &NodeDescriptor| match node {
        NodeDescriptor::Physical { storages, .. } => {
            execute_container_operation(dfs, storages, |backend, path| backend.create_dir(path))
                .ok_or(DfsFrontendError::StorageNotResponsive)
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
    };

    match nodes.len() {
        0 => Err(DfsFrontendError::ParentDoesNotExist),
        1 => create_dir_in_container(dfs, &nodes.pop().unwrap()),
        _ => {
            let mut nodes_that_have_parent: Vec<&NodeDescriptor> = nodes
                .iter()
                .filter_map(|node| {
                    let parent = node.parent()?;

                    match parent {
                        NodeDescriptor::Physical { storages, .. } => {
                            let exists =
                                execute_container_operation(dfs, &storages, |backend, path| {
                                    backend.path_exists(path)
                                })
                                .ok_or(DfsFrontendError::StorageNotResponsive);
                            match exists {
                                Ok(true) => Some(Ok(node)),
                                Ok(false) => None,
                                Err(e) => Some(Err(e)),
                            }
                        }
                        NodeDescriptor::Virtual { .. } => None,
                    }
                })
                .collect::<Result<_, DfsFrontendError>>()?;

            match nodes_that_have_parent.len() {
                0 => Err(DfsFrontendError::ParentDoesNotExist),
                1 => create_dir_in_container(dfs, nodes_that_have_parent.pop().unwrap()),
                _ => Err(DfsFrontendError::ReadOnlyPath), // Ambiguous path are for now read-only
            }
        }
    }
}

pub fn remove_dir(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
) -> Result<(), DfsFrontendError> {
    let requested_path = PathBuf::from_str(&requested_path).unwrap();
    let mut nodes = get_related_nodes(dfs, &requested_path)?;

    let remove_dir_from_container = |dfs: &mut UnencryptedDfs, node: &NodeDescriptor| match node {
        NodeDescriptor::Physical { storages, .. } => {
            execute_container_operation(dfs, storages, |backend, path| backend.remove_dir(path))
                .ok_or(DfsFrontendError::StorageNotResponsive)
                .and_then(|resp| match resp {
                    RemoveDirResponse::Removed => Ok(()),
                    RemoveDirResponse::NotFound => Err(DfsFrontendError::NoSuchPath),
                    RemoveDirResponse::NotADirectory => Err(DfsFrontendError::NotADirectory),
                    RemoveDirResponse::DirNotEmpty => Err(DfsFrontendError::DirNotEmpty),
                    RemoveDirResponse::RootRemovalNotAllowed => Err(DfsFrontendError::ReadOnlyPath),
                })
        }
        NodeDescriptor::Virtual { .. } => Err(DfsFrontendError::ReadOnlyPath),
    };

    match nodes.len() {
        0 => Err(DfsFrontendError::NoSuchPath),
        1 => remove_dir_from_container(dfs, &nodes.pop().unwrap()),
        _ => {
            let mut existent_paths: Vec<&NodeDescriptor> =
                fetch_data_from_containers(&nodes, dfs, |backend, path| backend.path_exists(path))
                    .filter_map(|(node, exists)| if exists { Some(node) } else { None })
                    .collect();

            match existent_paths.len() {
                0 => Err(DfsFrontendError::NoSuchPath),
                1 => remove_dir_from_container(dfs, existent_paths.pop().unwrap()),
                _ => Err(DfsFrontendError::ReadOnlyPath), // Ambiguous path are for now read-only
            }
        }
    }
}
