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
    storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    storage_backends: HashMap<Uuid, Rc<dyn StorageBackend>>, // Key: Storage uuid, Value: corresponding backend handling communication
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
    fn readdir<P: AsRef<Path>>(&mut self, path: P) -> Vec<NodeDescriptor> {
        let resolved_paths = self.path_resolver.resolve(path.as_ref());
        let nodes = resolved_paths
            .into_iter()
            .filter_map(|PathWithStorages { path, storages }| {
                let mut backends = storages.iter().flat_map(|storage| {
                    let backend = self.get_backend(storage);
                    // TODO for now unsupported backends are ignored
                    if backend.is_none() {
                        tracing::error!("Unsupported storage backend: {}", storage.backend_type());
                    }
                    backend.map(|backend| (backend, storage))
                }); // TODO is lack of backend a fatal error ? should we have some independent error channel ?

                // TODO getting first should be a temporary policy, maybe we should ping backends to check if any of them
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
                    tracing::error!(
                        "None of backend for storages {:?} works",
                        storages.iter().map(|s| s.backend_type())
                    );
                    None
                }
            })
            .flatten();

        // TODO check conflicts: more than one container may have nodes claiming the same path
        nodes.collect()
    }
}
