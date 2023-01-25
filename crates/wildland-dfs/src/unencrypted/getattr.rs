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

use itertools::Itertools;
use wildland_corex::dfs::interface::{DfsFrontendError, NodeType, Stat};
use wildland_corex::ResolvedPath;

use super::{
    execute_backend_op_with_policy,
    ExecutionPolicy,
    NodeDescriptor,
    NodeStorages,
    UnencryptedDfs,
};

pub fn getattr(
    dfs_front: &mut UnencryptedDfs,
    input_exposed_path: String,
) -> Result<Stat, DfsFrontendError> {
    let input_exposed_path = Path::new(&input_exposed_path);

    let requested_abs_path = dfs_front
        .path_translator
        .exposed_to_absolute_path(input_exposed_path);

    let resolved_paths = dfs_front.path_resolver.resolve(&requested_abs_path)?;

    let nodes = resolved_paths
        .into_iter()
        .map(|resolved_path| {
            map_resolved_path_into_node_descriptor(requested_abs_path.clone(), resolved_path)
        })
        .collect_vec();

    let mut stats: Vec<(&NodeDescriptor, Stat)> = nodes
        .iter()
        .filter_map(|node| {
            node.storages.as_ref().and_then(|storages| {
                fetch_data_from_backend(dfs_front, storages).map(|stat| (node, stat))
            })
        })
        .chain(nodes.iter().filter_map(|n| {
            if n.storages.is_none() {
                Some((
                    n,
                    Stat {
                        node_type: NodeType::Dir,
                        size: 0,
                        access_time: None,
                        modification_time: None,
                        change_time: None,
                    },
                ))
            } else {
                None
            }
        }))
        .collect();

    match stats.len() {
        0 => Err(DfsFrontendError::NoSuchPath),
        1 => Ok(stats.pop().unwrap().1),
        _ => {
            let nodes: Vec<&NodeDescriptor> = stats.into_iter().map(|(n, _)| n).collect();
            let exposed_paths = dfs_front.path_translator.solve_conflicts(nodes);
            let node = find_node_matching_requested_path(input_exposed_path, &exposed_paths);
            match node {
                // Physical node
                Some(NodeDescriptor {
                    storages: Some(node_storages),
                    ..
                }) => fetch_data_from_backend(dfs_front, node_storages)
                    .ok_or(DfsFrontendError::NoSuchPath),
                // Virtual node
                Some(_) | None if !exposed_paths.is_empty() => Ok(Stat {
                    node_type: NodeType::Dir,
                    size: 0,
                    access_time: None,
                    modification_time: None,
                    change_time: None,
                }),
                _ => Err(DfsFrontendError::NoSuchPath),
            }
        }
    }
}

fn fetch_data_from_backend(
    dfs_front: &mut UnencryptedDfs,
    node_storages: &NodeStorages,
) -> Option<Stat> {
    let backends = dfs_front.get_backends(&node_storages.storages);

    let backend_ops = backends.map(|backend| backend.getattr(&node_storages.path_within_storage));

    // TODO WILX-362
    execute_backend_op_with_policy(
        &node_storages.storages,
        backend_ops,
        ExecutionPolicy::SequentiallyToFirstSuccess,
    )
    .flatten()
}

fn find_node_matching_requested_path<'a>(
    input_exposed_path: &Path,
    exposed_paths: &[(&'a NodeDescriptor, PathBuf)],
) -> Option<&'a NodeDescriptor> {
    exposed_paths
        .iter()
        .find_map(|(node, exposed_path)| {
            if exposed_path == input_exposed_path {
                Some(node)
            } else {
                None
            }
        })
        .copied()
}

fn map_resolved_path_into_node_descriptor(
    requested_abs_path: PathBuf,
    resolved_path: ResolvedPath,
) -> NodeDescriptor {
    match resolved_path {
        ResolvedPath::PathWithStorages {
            path_within_storage,
            storages_id,
            storages,
        } => NodeDescriptor {
            storages: Some(NodeStorages::new(
                storages,
                path_within_storage,
                storages_id,
            )),
            absolute_path: requested_abs_path,
        },
        ResolvedPath::VirtualPath(_) => NodeDescriptor {
            storages: None,
            absolute_path: requested_abs_path,
        },
    }
}
