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

mod path_translator;
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

use self::path_translator::uuid_in_dir::UuidInDirTranslator;
use self::path_translator::PathTranslator;
use crate::storage_backend::{StorageBackend, StorageBackendError};

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
    path_translator: Box<dyn PathTranslator>,
}

impl UnencryptedDfs {
    pub fn new(
        path_resolver: Rc<dyn PathResolver>,
        storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    ) -> Self {
        Self {
            path_resolver: path_resolver.clone(),
            storage_backend_factories,
            storage_backends: HashMap::new(),
            path_translator: Box::new(UuidInDirTranslator::new(path_resolver)),
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
    fn readdir(&mut self, requested_path: String) -> Vec<String> {
        let requested_path = PathBuf::from_str(&requested_path).unwrap();
        let resolved_paths = self.path_resolver.resolve(requested_path.as_ref());
        let nodes = resolved_paths
            .into_iter()
            .filter_map(|resolved_path| {
                match resolved_path {
                    ResolvedPath::VirtualPath(virtual_path) => {
                        let virtual_nodes = self
                            .path_resolver
                            .list_virtual_nodes_in(&virtual_path)
                            .into_iter()
                            .map(|node_name| NodeDescriptor {
                                storages: None,
                                absolute_path: requested_path.join(node_name),
                            });
                        Some(Either::Left(virtual_nodes))
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
                                        Either::Left(resulting_paths.into_iter().map({
                                            let path = requested_path.clone();
                                            let storages = storages.clone();
                                            move |entry_path| NodeDescriptor {
                                                storages: Some(NodeStorages::new(
                                                    storages.clone(),
                                                    entry_path.clone(),
                                                    storages_id,
                                                )),
                                                absolute_path: path
                                                    .join(entry_path.file_name().unwrap()),
                                            }
                                        }))
                                    })
                                    .or_else(|err| match err {
                                        StorageBackendError::NotADirectory => {
                                            Ok(Either::Right(std::iter::once(NodeDescriptor {
                                                storages: Some(NodeStorages::new(
                                                    storages.clone(),
                                                    path_within_storage.clone(),
                                                    storages_id,
                                                )),
                                                absolute_path: requested_path.clone(),
                                            })))
                                        }
                                        _ => Err(err),
                                    })
                            }
                        });

                        let node_descriptors = execute_backend_op_with_policy(
                            &storages,
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

        self.path_translator
            .assign_exposed_paths(nodes)
            .into_iter()
            .filter_map(|(_node, exposed_path)| match exposed_path {
                Some(exposed_path) => {
                    if exposed_path.components().count() > requested_path.components().count() + 1 {
                        let mut parent = PathBuf::from(&exposed_path);
                        parent.pop();
                        Some(parent.to_string_lossy().to_string() + "/")
                    } else if exposed_path == requested_path {
                        None
                    } else {
                        Some(exposed_path.to_string_lossy().to_string())
                    }
                }
                None => None,
            })
            .unique()
            .collect()
    }

    fn getattr(&mut self, input_exposed_path: String) -> Option<Stat> {
        let input_exposed_path = Path::new(&input_exposed_path);
        let requested_abs_path = self
            .path_translator
            .exposed_to_absolute_path(input_exposed_path);

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
                },
            })
            .collect_vec();
        let node = self
            .path_translator
            .assign_exposed_paths(nodes)
            .into_iter()
            .filter_map(|(node, opt_exposed_path)| {
                opt_exposed_path.map(|exposed_path| (node, exposed_path))
            })
            .find_map(|(node, exposed_path)| {
                if exposed_path == input_exposed_path {
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
                    &node_storages.storages,
                    backend_ops,
                    ExecutionPolicy::SequentiallyToFirstSuccess,
                )
                .flatten()
            }
            // Virtual node
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
    storages: &[Storage],
    ops: impl Iterator<Item = Result<T, StorageBackendError>>,
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
                    "None of the backends for storages {:?} works",
                    storages.iter().map(|s| s.backend_type())
                );
                None
            })
        }
    }
}
