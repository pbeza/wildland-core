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
use std::str::FromStr;

use itertools::{Either, Itertools};
use uuid::Uuid;
use wildland_corex::{ResolvedPath, Storage};

use super::{
    execute_backend_op_with_policy,
    ExecutionPolicy,
    NodeDescriptor,
    NodeStorages,
    UnencryptedDfs,
};
use crate::storage_backend::StorageBackendError;

pub fn readdir(dfs_front: &mut UnencryptedDfs, requested_path: String) -> Vec<String> {
    let requested_path = PathBuf::from_str(&requested_path).unwrap();
    let resolved_paths = dfs_front.path_resolver.resolve(requested_path.as_ref());

    let nodes = resolved_paths
        .into_iter()
        .filter_map(|resolved_path| {
            map_resolved_path_into_node_descriptor(dfs_front, &requested_path, resolved_path)
        })
        .flatten()
        .collect_vec();

    dfs_front
        .path_translator
        .assign_exposed_paths(nodes)
        .into_iter()
        .filter_map(|(_node, exposed_path)| filter_exposed_paths(&requested_path, exposed_path))
        .unique()
        .collect()
}

fn filter_exposed_paths(requested_path: &Path, exposed_path: Option<PathBuf>) -> Option<String> {
    match exposed_path {
        Some(exposed_path) => {
            if exposed_path.components().count() > requested_path.components().count() + 1 {
                // filter out paths that have extended path with uuid at the end due to the conflicts
                let mut parent = PathBuf::from(&exposed_path);
                parent.pop();
                Some(parent.to_string_lossy().to_string() + "/")
            } else if exposed_path == requested_path {
                // filter out paths the same as the requested one
                // it may happen because of the conflicts resolution
                None
            } else {
                Some(exposed_path.to_string_lossy().to_string())
            }
        }
        // Some nodes may not have exposed path
        None => None,
    }
}

fn map_resolved_path_into_node_descriptor<'a>(
    dfs_front: &mut UnencryptedDfs,
    requested_path: &'a Path,
    resolved_path: ResolvedPath,
) -> Option<impl Iterator<Item = NodeDescriptor> + 'a> {
    match resolved_path {
        ResolvedPath::VirtualPath(virtual_path) => Some(Either::Left(
            map_virtual_path_to_node_descriptor(dfs_front, requested_path, &virtual_path),
        )),
        ResolvedPath::PathWithStorages {
            path_within_storage,
            storages_id,
            storages,
        } => map_physical_path_to_node_descriptor(
            dfs_front,
            requested_path,
            storages_id,
            storages,
            path_within_storage,
        )
        .map(Either::Right),
    }
}

fn map_virtual_path_to_node_descriptor<'a>(
    dfs_front: &mut UnencryptedDfs,
    requested_path: &'a Path,
    virtual_path: &Path,
) -> impl Iterator<Item = NodeDescriptor> + 'a {
    dfs_front
        .path_resolver
        .list_virtual_nodes_in(virtual_path)
        .into_iter()
        .map(|node_name| NodeDescriptor {
            storages: None,
            absolute_path: requested_path.join(node_name),
        })
}

fn map_physical_path_to_node_descriptor<'a>(
    dfs_front: &mut UnencryptedDfs,
    requested_path: &'a Path,
    storages_id: Uuid,
    node_storages: Vec<Storage>,
    path_within_storage: PathBuf,
) -> Option<impl Iterator<Item = NodeDescriptor> + 'a> {
    let backends = dfs_front.get_backends(&node_storages);

    let operations_on_backends = backends.map(|backend| {
        let node_storages = node_storages.clone();
        {
            backend
                .readdir(&path_within_storage)
                .map(|resulting_paths| {
                    Either::Left(resulting_paths.into_iter().map({
                        let node_storages = node_storages.clone();
                        move |entry_path| NodeDescriptor {
                            storages: Some(NodeStorages::new(
                                node_storages.clone(),
                                entry_path.clone(),
                                storages_id,
                            )),
                            absolute_path: requested_path.join(entry_path.file_name().unwrap()),
                        }
                    }))
                })
                .or_else(|err| match err {
                    StorageBackendError::NotADirectory => {
                        Ok(Either::Right(std::iter::once(NodeDescriptor {
                            storages: Some(NodeStorages::new(
                                node_storages.clone(),
                                path_within_storage.clone(),
                                storages_id,
                            )),
                            absolute_path: PathBuf::from(requested_path),
                        })))
                    }
                    _ => Err(err),
                })
        }
    });

    execute_backend_op_with_policy(
        &node_storages,
        operations_on_backends,
        // TODO WILX-362 getting first should be a temporary policy, maybe we should ping backends to check if any of them
        // is responsive and use the one that answered as the first one or ask all of them at once and return the first answer.
        ExecutionPolicy::SequentiallyToFirstSuccess,
    )
}
