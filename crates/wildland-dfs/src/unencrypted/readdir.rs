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

use itertools::Itertools;
use uuid::Uuid;
use wildland_corex::dfs::interface::DfsFrontendError;
use wildland_corex::{ResolvedPath, Storage};

use super::node_descriptor::{NodeDescriptor, NodeStorages};
use super::utils::{execute_backend_op_with_policy, ExecutionPolicy};
use super::UnencryptedDfs;
use crate::storage_backends::models::ReaddirResponse;

pub fn readdir(
    dfs_front: &mut UnencryptedDfs,
    requested_path: String,
) -> Result<Vec<String>, DfsFrontendError> {
    let requested_path = PathBuf::from_str(&requested_path).unwrap();
    let resolved_paths = dfs_front.path_resolver.resolve(requested_path.as_ref())?;

    let nodes = resolved_paths
        .into_iter()
        .map(|resolved_path| {
            map_resolved_path_into_node_descriptors(dfs_front, &requested_path, resolved_path)
        })
        .collect_vec();

    if nodes
        .iter()
        .all(|result| matches!(result, Err(DfsFrontendError::NoSuchPath)))
    {
        return Err(DfsFrontendError::NoSuchPath);
    };

    let nodes = nodes
        .into_iter()
        .filter_map(|result| if let Ok(r) = result { Some(r) } else { None })
        .flat_map(|v| v.into_iter())
        .collect_vec();

    Ok(dfs_front
        .path_translator
        .solve_conflicts(nodes.iter().collect::<Vec<_>>())
        .into_iter()
        .filter_map(|(_node, exposed_path)| filter_exposed_paths(&requested_path, exposed_path))
        .unique()
        .collect())
}

// This function probably can be removed in case of using different path translator than `uuid_in_dir`
fn filter_exposed_paths(requested_path: &Path, exposed_path: PathBuf) -> Option<String> {
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

fn map_resolved_path_into_node_descriptors(
    dfs_front: &mut UnencryptedDfs,
    requested_path: &Path,
    resolved_path: ResolvedPath,
) -> Result<Vec<NodeDescriptor>, DfsFrontendError> {
    match resolved_path {
        ResolvedPath::VirtualPath(virtual_path) => Ok(vec![map_virtual_path_to_node_descriptor(
            requested_path,
            &virtual_path,
        )]),
        ResolvedPath::PathWithStorages {
            path_within_storage,
            storages_id,
            storages,
        } => map_physical_path_to_node_descriptors(
            dfs_front,
            requested_path,
            storages_id,
            storages,
            path_within_storage,
        ),
    }
}

fn map_virtual_path_to_node_descriptor(
    requested_path: &Path,
    virtual_path: &Path,
) -> NodeDescriptor {
    let component_after_requested_path = virtual_path
        .strip_prefix(requested_path.to_str().unwrap())
        .unwrap()
        .components()
        .next();
    let absolute_path = match component_after_requested_path {
        Some(std::path::Component::Normal(component)) => requested_path.join(component),
        None => requested_path.into(),
        _ => panic!("There is a bug in PathResolver, probably it returned a path that does not start with the requested one"),
    };
    NodeDescriptor::Virtual { absolute_path }
}

fn map_physical_path_to_node_descriptors(
    dfs_front: &mut UnencryptedDfs,
    requested_path: &Path,
    storages_id: Uuid,
    node_storages: Vec<Storage>,
    path_within_storage: PathBuf,
) -> Result<Vec<NodeDescriptor>, DfsFrontendError> {
    let backends = dfs_front.get_backends(&node_storages);

    let operations_on_backends = backends.map(|backend| {
        let node_storages = node_storages.clone();
        {
            backend
                .readdir(&path_within_storage)
                .map(|response| match response {
                    ReaddirResponse::Entries(resulting_paths) => Ok(resulting_paths
                        .into_iter()
                        .map({
                            let node_storages = node_storages.clone();
                            move |entry_path| NodeDescriptor::Physical {
                                storages: NodeStorages::new(
                                    node_storages.clone(),
                                    entry_path.clone(),
                                    storages_id,
                                ),
                                absolute_path: requested_path.join(entry_path.file_name().unwrap()),
                            }
                        })
                        .collect_vec()),
                    ReaddirResponse::NotADirectory => Ok(vec![NodeDescriptor::Physical {
                        storages: NodeStorages::new(
                            node_storages.clone(),
                            path_within_storage.clone(),
                            storages_id,
                        ),
                        absolute_path: PathBuf::from(requested_path),
                    }]),
                    ReaddirResponse::NoSuchPath => Err(DfsFrontendError::NoSuchPath),
                })
        }
    });

    execute_backend_op_with_policy(
        &node_storages,
        operations_on_backends,
        // TODO WILX-362 getting first should be a temporary policy, maybe we should ping backends to check if any of them
        // is responsive and use the one that answered as the first one or ask all of them at once and return the first answer.
        ExecutionPolicy::SequentiallyToFirstSuccess,
    )?
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(
        PathBuf::from("/a/b"),
        PathBuf::from("/a/b/c"),
        PathBuf::from("/a/b/c")
    )]
    #[test_case(PathBuf::from("/a"), PathBuf::from("/a/b"), PathBuf::from("/a/b"))]
    #[test_case(PathBuf::from("/a"), PathBuf::from("/a/b/c"), PathBuf::from("/a/b"))]
    #[test_case(PathBuf::from("/"), PathBuf::from("/a/b/"), PathBuf::from("/a"))]
    #[test_case(PathBuf::from("/"), PathBuf::from("/a/b/c"), PathBuf::from("/a"))]
    fn test_map_virtual_path_to_node_descriptor(
        requested_path: PathBuf,
        resolved_virtual_path: PathBuf,
        expected_node_path: PathBuf,
    ) {
        let node_descriptor =
            map_virtual_path_to_node_descriptor(&requested_path, &resolved_virtual_path);
        assert_eq!(
            node_descriptor,
            NodeDescriptor::Virtual {
                absolute_path: expected_node_path
            }
        );
    }
}
