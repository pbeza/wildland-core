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
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use itertools::Either;
use uuid::Uuid;
use wildland_corex::dfs::interface::{DfsFrontend, NodeDescriptor, NodeStorage};
use wildland_corex::{PathResolver, ResolvedPath, Storage};

use crate::storage_backend::StorageBackend;

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
    fn assign_backends_to_storages<'a>(
        &'a mut self,
        storages: &'a [Storage],
    ) -> impl Iterator<Item = (Rc<dyn StorageBackend>, &Storage)> + '_ {
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
                Ok(backend) => Some((backend, storage)),
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
    fn readdir(&mut self, path: String) -> Vec<NodeDescriptor> {
        let path = PathBuf::from_str(&path).unwrap();
        let resolved_paths = self.path_resolver.resolve(path.as_ref());
        let nodes = resolved_paths
            .into_iter()
            .filter_map(|resolved_path| {
                match resolved_path {
                    // None means virtual node, for example node /a/ when some container claims a path like /a/b/
                    ResolvedPath::VirtualPath(virtual_path) => {
                        Some(Either::Left(std::iter::once(NodeDescriptor {
                            storage: None,
                            absolute_path: path.join(virtual_path.file_name().unwrap()),
                        })))
                    }
                    ResolvedPath::PathWithStorages {
                        path_within_storage,
                        storages,
                    } => {
                        let backends_with_storages = self.assign_backends_to_storages(&storages);

                        let operations_on_backends =
                            backends_with_storages.map(|(backend, storage)| {
                                backend
                                    .readdir(&path_within_storage)
                                    .map(|resulting_paths| {
                                        resulting_paths.into_iter().map({
                                            let storage = storage.clone();
                                            let path = path.clone();
                                            move |entry_path| NodeDescriptor {
                                                storage: Some(NodeStorage::new(
                                                    storage.clone(),
                                                    entry_path.clone(),
                                                )),
                                                absolute_path: path
                                                    .join(entry_path.file_name().unwrap()),
                                            }
                                        })
                                    })
                            });

                        let node_descriptors = execute_backend_op_with_policy(
                            operations_on_backends,
                            // TODO WILX-362 getting first should be a temporary policy, maybe we should ping backends to check if any of them
                            // is responsive and use the one that answered as the first one or ask all of them at once and return the first answer.
                            ExecutionPolicy::SequentiallyToFirstSuccess,
                        );

                        if node_descriptors.is_none() {
                            // TODO WILX-363 send alert to the wildland app bypassing DFS Frontend API
                            tracing::error!(
                                "None of the backends for storages {:?} works",
                                storages.iter().map(|s| s.backend_type())
                            );
                        }
                        node_descriptors.map(Either::Right)
                    }
                }
            })
            .flatten();

        nodes.collect()
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
        }
    }
}
