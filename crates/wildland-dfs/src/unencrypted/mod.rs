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

#[cfg(test)]
mod tests;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use itertools::{Either, Itertools};
use uuid::Uuid;
use wildland_corex::dfs::interface::{DfsFrontend, NodeType, Stat};
use wildland_corex::{PathResolver, ResolvedPath, Storage};

use crate::storage_backend::StorageBackend;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeDescriptor {
    pub storages: Option<NodeStorages>, // nodes may not have storage - so called virtual nodes
    pub absolute_path: PathBuf,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeStorages {
    storages: Vec<Storage>,
    path_within_storage: PathBuf,
    uuid: Uuid,
}

impl NodeStorages {
    pub fn new(storages: Vec<Storage>, path_within_storage: PathBuf, uuid: Uuid) -> Self {
        Self {
            storages,
            path_within_storage,
            uuid,
        }
    }

    pub fn path_within_storage(&self) -> &Path {
        &self.path_within_storage
    }
}

pub trait StorageBackendFactory {
    fn init_backend(&self, storage: Storage) -> Result<Rc<dyn StorageBackend>, anyhow::Error>;
}

pub struct UnencryptedDfs {
    path_resolver: Rc<dyn PathResolver>,
    /// Stores a factory for each supported backend type
    storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    /// Stores Backend for each storage. Each storage should have its own backend cause even within
    /// a single backend type (like S3) each storage may point to a different location (like S3 bucket).
    /// It is up to StorageBackend and StorageBackendFactory implementation whether all backends of a
    /// given type reuse some connector/client (factory could initiate each backend with some shared
    /// reference).
    storage_backends: HashMap<Uuid, Rc<dyn StorageBackend>>,
}

impl UnencryptedDfs {
    pub fn new(
        path_resolver: Rc<dyn PathResolver>,
        storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    ) -> Self {
        Self {
            path_resolver,
            storage_backend_factories,
            storage_backends: HashMap::new(),
        }
    }

    /// Returns StorageBackend for a given Storage.
    ///
    /// If there is no StorageBackend for a given Storage, StorageBackendFactory, related to its
    /// type, is used to create one.
    fn get_backend(
        &mut self,
        storage: &Storage,
    ) -> Result<Rc<dyn StorageBackend>, Box<dyn std::error::Error>> {
        let backend = match self.storage_backends.entry(storage.uuid()) {
            Entry::Occupied(occupied_entry) => occupied_entry.get().clone(),
            Entry::Vacant(vacant_entry) => {
                match self.storage_backend_factories.get(storage.backend_type()) {
                    Some(factory) => {
                        let backend = factory.init_backend(storage.clone())?;
                        vacant_entry.insert(backend).clone()
                    }
                    None => {
                        return Err(Box::<dyn std::error::Error>::from(format!(
                            "Could not find backend factory for {} storage",
                            storage.backend_type()
                        )))
                    }
                }
            }
        };

        Ok(backend)
    }

    /// Matches every given Storage with its StorageBackend and return iterator over such pairs.
    fn get_backends<'a>(
        &'a mut self,
        storages: &'a [Storage],
    ) -> impl Iterator<Item = Rc<dyn StorageBackend>> + '_ {
        storages.iter().filter_map(|storage| {
            match self.get_backend(storage) {
                Err(e) => {
                    // TODO WILX-363 send alert to the wildland app bypassing DFS Frontend API
                    tracing::error!(
                        "Unsupported storage backend: {}; Reason: {}",
                        storage.backend_type(),
                        e
                    );
                    None
                }
                Ok(backend) => Some(backend),
            }
        })
    }

    // Extract as a trait
    // TODO docs
    // make sure that exposed paths are unique
    fn assign_exposed_paths(
        &self,
        nodes: Vec<NodeDescriptor>,
    ) -> Vec<(NodeDescriptor, Option<PathBuf>)> {
        let counted_abs_paths = nodes.iter().counts_by(|node| node.absolute_path.clone());

        nodes
            .into_iter()
            .map(|node| {
                let abs_path = node.absolute_path.clone();
                if counted_abs_paths.get(&abs_path).unwrap() > &1 {
                    let exposed_path =
                        abs_path.join(node.storages.as_ref().map_or(PathBuf::new(), |s| {
                            PathBuf::from_str(s.uuid.to_string().as_str()).unwrap()
                        }));
                    (node, Some(exposed_path))
                } else {
                    (node, Some(abs_path))
                }
            })
            .collect()
    }

    /// TODO description
    /// abs paths may collide
    /// exposed should be generated deterministically
    fn exposed_to_absolute_path(&self, path: &Path) -> PathBuf {
        match path.file_name() {
            Some(file_name) if Uuid::parse_str(&file_name.to_string_lossy()).is_ok() => {
                let mut path = PathBuf::from(path);
                path.pop();
                path
            }
            _ => path.into(),
        }
    }
}

impl DfsFrontend for UnencryptedDfs {
    /// Returns nodes descriptors found in the given path taking into account all of the user's
    /// containers.
    ///
    /// **NOTE: Conflicting paths**
    /// More than one container may have nodes claiming the same "full path" - meaning
    /// concatenation of a path claimed by a container with a path of the file inside the container.
    /// It is possible that returned nodes include both such files.
    /// Example:
    /// Container C1 claims path `/a` and includes file `/b/c`.
    /// Container C2 claims path `/a/b/` and includes file `/c`.
    /// In this case, result includes the following descriptors:
    /// [
    ///     NodeDescriptor { path: "/b/c", storage: <C1 storage>},
    ///     NodeDescriptor { path: "/c", storage: <C2 storage>},
    /// ]
    /// Full path within the user's forest for both nodes is `/a/b/c`. It is up to FS frontend how to
    /// show it to a user (e.g. by prefixing it with some storage-specific tag).
    fn readdir(&mut self, path: String) -> Vec<String> {
        let path = PathBuf::from_str(&path).unwrap();
        let resolved_paths = self.path_resolver.resolve(path.as_ref());
        let nodes = resolved_paths
            .into_iter()
            .filter_map(|resolved_path| {
                match resolved_path {
                    ResolvedPath::VirtualPath(virtual_path) => {
                        Some(Either::Left(std::iter::once(NodeDescriptor {
                            storages: None,
                            absolute_path: path.join(virtual_path.file_name().unwrap()),
                        })))
                    }
                    ResolvedPath::PathWithStorages {
                        path_within_storage,
                        storages_id,
                        storages,
                    } => {
                        let backends = self.get_backends(&storages);

                        let operations_on_backends = backends.map(|backend| {
                            let storages = storages.clone();
                            {
                                backend
                                    .readdir(&path_within_storage)
                                    .map(|resulting_paths| {
                                        resulting_paths.into_iter().map({
                                            let path = path.clone();
                                            move |entry_path| NodeDescriptor {
                                                storages: Some(NodeStorages::new(
                                                    storages.clone(),
                                                    entry_path.clone(),
                                                    storages_id,
                                                )),
                                                absolute_path: path
                                                    .join(entry_path.file_name().unwrap()),
                                            }
                                        })
                                    })
                            }
                        });

                        let node_descriptors = execute_backend_op_with_policy(
                            operations_on_backends,
                            // TODO WILX-362 getting first should be a temporary policy, maybe we should ping backends to check if any of them
                            // is responsive and use the one that answered as the first one or ask all of them at once and return the first answer.
                            ExecutionPolicy::SequentiallyToFirstSuccess,
                        );

                        node_descriptors.map(Either::Right)
                    }
                }
            })
            .flatten()
            .collect_vec();

        self.assign_exposed_paths(nodes)
            .into_iter()
            .filter_map(|(_node, exposed_path)| {
                exposed_path.map(|p| p.to_string_lossy().to_string())
            })
            .collect()
    }

    fn getattr(&mut self, requested_path: String) -> Option<Stat> {
        let requested_abs_path = self.exposed_to_absolute_path(Path::new(&requested_path));

        let resolved_paths = self.path_resolver.resolve(&requested_abs_path);
        let nodes = resolved_paths
            .into_iter()
            .map(|resolved_path| match resolved_path {
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
                    absolute_path: requested_abs_path.clone(),
                },
                ResolvedPath::VirtualPath(_) => NodeDescriptor {
                    storages: None,
                    absolute_path: requested_abs_path.clone(),
                }, // TODO
            })
            .collect_vec();
        let node = self
            .assign_exposed_paths(nodes)
            .into_iter()
            .filter_map(|(node, opt_exposed_path)| {
                opt_exposed_path.map(|exposed_path| (node, exposed_path))
            })
            .find_map(|(node, exposed_path)| {
                if exposed_path == requested_abs_path {
                    Some(node)
                } else {
                    None
                }
            })?;

        match node.storages {
            Some(node_storages) => {
                let backends = self.get_backends(&node_storages.storages);

                let backend_ops =
                    backends.map(|backend| backend.getattr(&node_storages.path_within_storage));

                // TODO WILX-362
                execute_backend_op_with_policy(
                    backend_ops,
                    ExecutionPolicy::SequentiallyToFirstSuccess,
                )
                .flatten()
            }
            None => Some(Stat {
                node_type: NodeType::Dir,
            }),
        }
    }
}

enum ExecutionPolicy {
    SequentiallyToFirstSuccess,
}
fn execute_backend_op_with_policy<T: std::fmt::Debug>(
    ops: impl Iterator<Item = Result<T, anyhow::Error>>,
    policy: ExecutionPolicy,
) -> Option<T> {
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
            .map(Result::unwrap)
            .or_else(|| {
                // TODO WILX-363 send alert to the wildland app bypassing DFS Frontend API
                tracing::error!(
                    "ERROR TODO" // "None of the backends for storages {:?} works", // TODO
                                 // storages.iter().map(|s| s.backend_type())
                );
                None
            })
        }
    }
}
