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

mod getattr;
mod path_translator;
mod readdir;
#[cfg(test)]
mod tests;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use uuid::Uuid;
use wildland_corex::dfs::interface::{DfsFrontend, DfsFrontendError, Stat};
use wildland_corex::{PathResolver, Storage};

use self::path_translator::uuid_in_dir::UuidInDirTranslator;
use self::path_translator::PathConflictResolver;
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

// TODO WILX-387 Current DFS implementation uses some kind of mapping paths into another ones in order to
// avoid conflicts. There is always a probability that mapped path will be in conflict with some
// other user's file or directory (now we assume that user won't have any file named as an uuid
// string format). The problem could be solved by checking mapped path (e.g. containing uuid) if
// it represents a file/dir in the first place, and then, if no results are found, to check if
// the conflict resolution took place and find files which paths were mapped.

pub struct UnencryptedDfs {
    path_resolver: Box<dyn PathResolver>,
    /// Stores a factory for each supported backend type
    storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    /// Stores Backend for each storage. Each storage should have its own backend cause even within
    /// a single backend type (like S3) each storage may point to a different location (like S3 bucket).
    /// It is up to StorageBackend and StorageBackendFactory implementation whether all backends of a
    /// given type reuse some connector/client (factory could initiate each backend with some shared
    /// reference).
    storage_backends: HashMap<Uuid, Rc<dyn StorageBackend>>,
    path_translator: Box<dyn PathConflictResolver>,
}

impl UnencryptedDfs {
    pub fn new(
        path_resolver: Box<dyn PathResolver>,
        storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    ) -> Self {
        Self {
            path_resolver,
            storage_backend_factories,
            storage_backends: HashMap::new(),
            path_translator: Box::new(UuidInDirTranslator::new()),
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
    fn readdir(&mut self, requested_path: String) -> Result<Vec<String>, DfsFrontendError> {
        readdir::readdir(self, requested_path)
    }

    // Returns Stat of the file indicated by the provided exposed path
    fn getattr(&mut self, input_exposed_path: String) -> Result<Stat, DfsFrontendError> {
        getattr::getattr(self, input_exposed_path)
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
