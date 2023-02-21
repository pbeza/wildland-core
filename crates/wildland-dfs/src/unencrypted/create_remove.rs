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

use wildland_corex::dfs::interface::{DfsFrontendError, FileHandle};

use super::node_descriptor::NodeStorages;
use super::utils::{exec_on_single_existing_node, execute_container_operation, get_related_nodes};
use super::{NodeDescriptor, UnencryptedDfs};
use crate::storage_backends::models::{
    CreateDirResponse,
    CreateFileResponse,
    RemoveDirResponse,
    RemoveFileResponse,
};

pub fn create_dir(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
) -> Result<(), DfsFrontendError> {
    let create_dir_in_container = |dfs: &mut UnencryptedDfs, storages: &NodeStorages| {
        execute_container_operation(dfs, storages, &|backend| {
            backend.create_dir(storages.path_within_storage())
        })
        .and_then(|resp| match resp {
            CreateDirResponse::Created => Ok(()),
            CreateDirResponse::InvalidParent => Err(DfsFrontendError::InvalidParent),
            CreateDirResponse::PathAlreadyExists => Err(DfsFrontendError::PathAlreadyExists),
        })
    };

    generic_create(dfs, requested_path, create_dir_in_container)
}

pub fn create_file(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
) -> Result<FileHandle, DfsFrontendError> {
    let create_file_in_container = |dfs: &mut UnencryptedDfs, storages: &NodeStorages| {
        execute_container_operation(dfs, storages, &|backend| {
            backend.create_file(storages.path_within_storage())
        })
        .and_then(|resp| match resp {
            CreateFileResponse::Created(opened_file) => Ok(dfs.insert_opened_file(opened_file)),
            CreateFileResponse::ParentDoesNotExist => Err(DfsFrontendError::InvalidParent),
        })
    };

    generic_create(dfs, requested_path, create_file_in_container)
}

fn generic_create<T>(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
    container_op: fn(&mut UnencryptedDfs, &NodeStorages) -> Result<T, DfsFrontendError>,
) -> Result<T, DfsFrontendError> {
    let create_in_container = |dfs: &mut UnencryptedDfs, node: &NodeDescriptor| match node {
        NodeDescriptor::Physical { storages, .. } => container_op(dfs, storages),
        NodeDescriptor::Virtual { .. } => Err(DfsFrontendError::PathAlreadyExists),
    };

    let requested_path = PathBuf::from_str(&requested_path).unwrap();
    let nodes = get_related_nodes(dfs, &requested_path)?;

    match nodes.as_slice() {
        [] => Err(DfsFrontendError::InvalidParent),
        [node] => create_in_container(dfs, node),
        _ => {
            let nodes_that_have_parent: Vec<_> = nodes
                .iter()
                .filter_map(|node| {
                    let parent = node.parent()?;

                    match parent {
                        NodeDescriptor::Physical { storages, .. } => {
                            let exists = execute_container_operation(dfs, &storages, &|backend| {
                                backend.path_exists(storages.path_within_storage())
                            });
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

            match nodes_that_have_parent.as_slice() {
                [] => Err(DfsFrontendError::InvalidParent),
                [node] => create_in_container(dfs, node),
                _ => Err(DfsFrontendError::ReadOnlyPath), // Ambiguous path are for now read-only
            }
        }
    }
}

pub fn remove_dir(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
) -> Result<(), DfsFrontendError> {
    let remove_dir_from_container = |dfs: &mut UnencryptedDfs, storages: &NodeStorages| {
        execute_container_operation(dfs, storages, &|backend| {
            backend.remove_dir(storages.path_within_storage())
        })
        .and_then(|resp| match resp {
            RemoveDirResponse::Removed => Ok(()),
            RemoveDirResponse::NotFound => Err(DfsFrontendError::NoSuchPath),
            RemoveDirResponse::NotADirectory => Err(DfsFrontendError::NotADirectory),
            RemoveDirResponse::DirNotEmpty => Err(DfsFrontendError::DirNotEmpty),
            RemoveDirResponse::RootRemovalNotAllowed => Err(DfsFrontendError::ReadOnlyPath),
        })
    };

    generic_remove(dfs, requested_path, remove_dir_from_container)
}

pub fn remove_file(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
) -> Result<(), DfsFrontendError> {
    let remove_file_from_container = |dfs: &mut UnencryptedDfs, storages: &NodeStorages| {
        execute_container_operation(dfs, storages, &|backend| {
            backend.remove_file(storages.path_within_storage())
        })
        .and_then(|resp| match resp {
            RemoveFileResponse::Removed => Ok(()),
            RemoveFileResponse::NotFound => Err(DfsFrontendError::NoSuchPath),
            RemoveFileResponse::NotAFile => Err(DfsFrontendError::NotAFile),
        })
    };

    generic_remove(dfs, requested_path, remove_file_from_container)
}

fn generic_remove<T>(
    dfs: &mut UnencryptedDfs,
    requested_path: String,
    container_op: fn(&mut UnencryptedDfs, &NodeStorages) -> Result<T, DfsFrontendError>,
) -> Result<T, DfsFrontendError> {
    let remove_from_container = |dfs: &mut UnencryptedDfs, node: &NodeDescriptor| {
        match node {
            NodeDescriptor::Physical { storages, .. } => container_op(dfs, storages), // call operation on physical nodes
            NodeDescriptor::Virtual { .. } => Err(DfsFrontendError::ReadOnlyPath), // Virtual paths are considered as read-only
        }
    };

    let requested_path = PathBuf::from_str(&requested_path).unwrap();
    let mut nodes = get_related_nodes(dfs, &requested_path)?;

    exec_on_single_existing_node(dfs, &mut nodes, &remove_from_container)
}
