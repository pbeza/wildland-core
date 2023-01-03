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

use crate::storage_backend::StorageBackend;
use std::{
    collections::{hash_map::Entry, HashMap},
    path::Path,
    rc::Rc,
};
use uuid::Uuid;
use wildland_corex::{
    dfs::interface::{DfsFrontend, NodeDescriptor},
    PathResolver, PathWithStorages, Storage,
};

pub trait StorageBackendFactory {
    fn init_backend(&self, storage: Storage) -> Rc<dyn StorageBackend>;
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

    fn get_backend(&mut self, storage: &Storage) -> Option<Rc<dyn StorageBackend>> {
        let backend_entry = self.storage_backends.entry(storage.uuid());
        let result = match backend_entry {
            Entry::Occupied(occupied_entry) => occupied_entry.get().clone(),
            Entry::Vacant(vacant_entry) => {
                let new_backend = self
                    .storage_backend_factories
                    .get(storage.backend_type())?
                    .init_backend(storage.clone());
                vacant_entry.insert(new_backend).clone()
            }
        };
        Some(result)
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
    fn readdir<P: AsRef<Path>>(&mut self, path: P) -> Vec<NodeDescriptor> {
        let resolved_paths = self.path_resolver.resolve(path.as_ref());
        let nodes = resolved_paths
            .into_iter()
            .filter_map(|PathWithStorages { path, storages }| {
                let mut backends = storages.iter().flat_map(|storage| {
                    let backend = self.get_backend(storage);
                    if backend.is_none() {
                        // TODO WILX-363 send alert to the wildland app bypassing DFS Frontend API
                        tracing::error!("Unsupported storage backend: {}", storage.backend_type());
                    }
                    backend.map(|backend| (backend, storage))
                });

                // TODO WILX-362 getting first should be a temporary policy, maybe we should ping backends to check if any of them
                // is responsive and use the one that answered as the first one or ask all of them at once and return the first answer.
                if let Some((backend, storage)) = backends.next() {
                    Some(backend.readdir(&path).into_iter().map({
                        let storage = storage.clone();
                        move |path| NodeDescriptor {
                            storage: storage.clone(),
                            path,
                        }
                    }))
                } else {
                    // TODO WILX-363 send alert to the wildland app bypassing DFS Frontend API
                    tracing::error!(
                        "None of the backends for storages {:?} works",
                        storages.iter().map(|s| s.backend_type())
                    );
                    None
                }
            })
            .flatten();

        nodes.collect()
    }
}
