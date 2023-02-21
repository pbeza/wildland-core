use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use itertools::Itertools;
use wildland_corex::dfs::interface::DfsFrontendError;
use wildland_corex::{ResolvedPath, Storage};

use super::node_descriptor::NodeStorages;
use super::{NodeDescriptor, UnencryptedDfs};
use crate::storage_backends::models::StorageBackendError;
use crate::storage_backends::StorageBackend;

pub fn filter_existent_nodes<'a: 'b, 'b>(
    nodes: &'a [NodeDescriptor],
    dfs: &'b mut UnencryptedDfs,
) -> Result<impl Iterator<Item = &'a NodeDescriptor>, DfsFrontendError> {
    Ok(nodes
        .iter()
        .map(move |node| match node {
            NodeDescriptor::Physical { storages, .. } => {
                execute_container_operation(dfs, storages, &|backend| {
                    backend.path_exists(storages.path_within_storage())
                })
                .map(|exists| exists.then_some(node))
            }
            NodeDescriptor::Virtual { .. } => Ok(Some(node)), // virtual nodes are forwarded
        })
        .collect::<Result<Vec<_>, DfsFrontendError>>()?
        .into_iter()
        .flatten())
}

pub fn execute_container_operation<T: Debug>(
    dfs_front: &mut UnencryptedDfs,
    node_storages: &NodeStorages,
    backend_op: &dyn Fn(Rc<dyn StorageBackend>) -> Result<T, StorageBackendError>,
) -> Result<T, DfsFrontendError> {
    let backends = dfs_front.get_backends(node_storages.storages());

    let backend_ops = backends.map(|backend| backend_op(backend));

    // TODO WILX-362
    execute_backend_op_with_policy(
        node_storages.storages(),
        backend_ops,
        ExecutionPolicy::SequentiallyToFirstSuccess,
    )
}

pub fn get_related_nodes(
    dfs_front: &mut UnencryptedDfs,
    input_exposed_path: &Path,
) -> Result<Vec<NodeDescriptor>, DfsFrontendError> {
    let requested_abs_path = dfs_front
        .path_translator
        .exposed_to_absolute_path(input_exposed_path);

    let resolved_paths = dfs_front.path_resolver.resolve(&requested_abs_path)?;

    Ok(resolved_paths
        .into_iter()
        .map(|resolved_path| {
            map_resolved_path_into_node_descriptor(requested_abs_path.clone(), resolved_path)
        })
        .collect_vec())
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
        } => NodeDescriptor::Physical {
            storages: NodeStorages::new(storages, path_within_storage, storages_id),
            absolute_path: requested_abs_path,
        },
        ResolvedPath::VirtualPath(_) => NodeDescriptor::Virtual {
            absolute_path: requested_abs_path,
        },
    }
}

pub enum ExecutionPolicy {
    SequentiallyToFirstSuccess,
}
pub fn execute_backend_op_with_policy<T: std::fmt::Debug>(
    storages: &[Storage],
    ops: impl Iterator<Item = Result<T, StorageBackendError>>,
    policy: ExecutionPolicy,
) -> Result<T, DfsFrontendError> {
    match policy {
        ExecutionPolicy::SequentiallyToFirstSuccess => {
            ops.inspect(|result| {
                if result.is_err() {
                    // TODO WILX-363 send alert to the wildland app bypassing DFS Frontend API
                    tracing::error!(
                        "Backend returned error for operation: {}",
                        result.as_ref().unwrap_err()
                    );
                }
            })
            .find(Result::is_ok)
            .map_or_else(
                || {
                    // TODO WILX-363 send alert to the wildland app bypassing DFS Frontend API
                    tracing::error!(
                        "None of the backends for storages {:?} works",
                        storages.iter().map(|s| s.backend_type())
                    );
                    Err(DfsFrontendError::StorageNotResponsive)
                },
                |r| Ok(r.unwrap()),
            )
        }
    }
}

pub fn find_node_matching_requested_path<'a>(
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

pub fn exec_on_single_existing_node<T>(
    dfs: &mut UnencryptedDfs,
    nodes: &mut Vec<NodeDescriptor>,
    operation: &dyn Fn(&mut UnencryptedDfs, &NodeDescriptor) -> Result<T, DfsFrontendError>,
) -> Result<T, DfsFrontendError> {
    match nodes.as_slice() {
        [] => Err(DfsFrontendError::NoSuchPath),
        [node] => operation(dfs, node),
        _ => {
            let existent_paths: Vec<_> = filter_existent_nodes(nodes, dfs)?.collect();

            match existent_paths.as_slice() {
                [] => Err(DfsFrontendError::NoSuchPath),
                [node] => operation(dfs, node),
                _ => Err(DfsFrontendError::ReadOnlyPath), // Ambiguous path are for now read-only
            }
        }
    }
}
