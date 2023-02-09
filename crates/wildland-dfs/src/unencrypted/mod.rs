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

mod create_remove_dir;
mod getattr;
mod path_translator;
mod readdir;
#[cfg(test)]
mod tests;
mod utils;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use uuid::Uuid;
use wildland_corex::dfs::interface::{DfsFrontend, DfsFrontendError, FileHandle, Stat};
use wildland_corex::{PathResolver, Storage};

use self::path_translator::uuid_in_dir::UuidInDirTranslator;
use self::path_translator::PathConflictResolver;
use self::utils::{fetch_data_from_containers, get_related_nodes};
use crate::storage_backends::models::{CloseError, OpenResponse, SeekFrom};
use crate::storage_backends::{
    CloseOnDropDescriptor,
    OpenedFileDescriptor,
    StorageBackend,
    StorageBackendFactory,
};
use crate::unencrypted::utils::find_node_matching_requested_path;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeDescriptor {
    Physical {
        storages: NodeStorages,
        absolute_path: PathBuf,
    },
    Virtual {
        absolute_path: PathBuf,
    },
}

impl NodeDescriptor {
    pub fn abs_path(&self) -> &Path {
        match self {
            NodeDescriptor::Physical { absolute_path, .. }
            | NodeDescriptor::Virtual { absolute_path } => absolute_path,
        }
    }

    pub fn is_physical(&self) -> bool {
        match self {
            NodeDescriptor::Virtual { .. } => false,
            NodeDescriptor::Physical { .. } => true,
        }
    }

    pub fn is_virtual(&self) -> bool {
        !self.is_physical()
    }

    pub fn storages(&self) -> Option<&NodeStorages> {
        match self {
            NodeDescriptor::Physical { storages, .. } => Some(storages),
            NodeDescriptor::Virtual { .. } => None,
        }
    }
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

// TODO WILX-387 Current DFS implementation uses some kind of mapping paths into another ones in order to
// avoid conflicts. There is always a probability that mapped path will be in conflict with some
// other user's file or directory (now we assume that user won't have any file named as an uuid
// string format). The problem could be solved by checking mapped path (e.g. containing uuid) if
// it represents a file/dir in the first place, and then, if no results are found, to check if
// the conflict resolution took place and find files which paths were mapped.

pub struct UnencryptedDfs {
    open_files: HashMap<Uuid, CloseOnDropDescriptor>,

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
            open_files: Default::default(),
            path_resolver,
            storage_backend_factories,
            storage_backends: HashMap::new(),
            path_translator: Box::new(UuidInDirTranslator::new()),
        }
    }

    fn insert_opened_file(&mut self, opened_file: CloseOnDropDescriptor) -> FileHandle {
        let uuid = Uuid::new_v4();
        self.open_files.insert(uuid, opened_file);
        FileHandle {
            descriptor_uuid: uuid,
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

    fn seek(&mut self, file: &FileHandle, seek_from: SeekFrom) -> Result<usize, DfsFrontendError> {
        if let Some(opened_file) = self.open_files.get_mut(&file.descriptor_uuid) {
            opened_file.seek(seek_from)
        } else {
            Err(DfsFrontendError::FileAlreadyClosed)
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
    fn readdir(&mut self, requested_path: String) -> Result<Vec<String>, DfsFrontendError> {
        readdir::readdir(self, requested_path)
    }

    // Returns Stat of the file indicated by the provided exposed path
    fn getattr(&mut self, input_exposed_path: String) -> Result<Stat, DfsFrontendError> {
        getattr::getattr(self, input_exposed_path)
    }

    fn open(&mut self, input_exposed_path: String) -> Result<FileHandle, DfsFrontendError> {
        let input_exposed_path = Path::new(&input_exposed_path);

        let nodes = get_related_nodes(self, input_exposed_path)?;

        let mut descriptors: Vec<(&NodeDescriptor, OpenResponse)> =
            fetch_data_from_containers(&nodes, self, |backend, path| backend.open(path))
                .collect::<Result<Vec<_>, DfsFrontendError>>()?
                .into_iter()
                .filter(|(_node, response)| !matches!(response, OpenResponse::NotFound))
                .collect();

        let map_response_to_result = |dfs_front: &mut UnencryptedDfs, resp: OpenResponse| match resp
        {
            OpenResponse::Found(opened_file) => Ok(dfs_front.insert_opened_file(opened_file)),
            OpenResponse::NotAFile => Err(DfsFrontendError::NotAFile),
            _ => Err(DfsFrontendError::NoSuchPath),
        };

        match descriptors.len() {
            0 => Err(DfsFrontendError::NoSuchPath),
            1 => map_response_to_result(self, descriptors.pop().unwrap().1),
            _ => {
                // More that 1 descriptor means that files are in conflict, so they are exposed under different paths
                let nodes: Vec<&NodeDescriptor> = descriptors.iter().map(|(n, _)| *n).collect();
                let exposed_paths = self.path_translator.solve_conflicts(nodes);
                let node = find_node_matching_requested_path(input_exposed_path, &exposed_paths);
                match node {
                    // Physical node
                    Some(node @ NodeDescriptor::Physical { .. }) => {
                        let opened_file = descriptors
                            .into_iter()
                            .find_map(|(n, resp)| if n == node { Some(resp) } else { None })
                            .ok_or(DfsFrontendError::NoSuchPath)?;
                        map_response_to_result(self, opened_file)
                    }
                    // Virtual node
                    Some(_) | None if !exposed_paths.is_empty() => Err(DfsFrontendError::NotAFile),
                    _ => Err(DfsFrontendError::NoSuchPath),
                }
            }
        }
    }

    fn close(&mut self, file: &FileHandle) -> Result<(), DfsFrontendError> {
        if let Some(opened_file) = self.open_files.remove(&file.descriptor_uuid) {
            opened_file.close().map_err(|e| match e {
                CloseError::FileAlreadyClosed => DfsFrontendError::FileAlreadyClosed,
            })
        } else {
            Err(DfsFrontendError::FileAlreadyClosed)
        }
    }

    fn create_dir(&mut self, requested_path: String) -> Result<(), DfsFrontendError> {
        create_remove_dir::create_dir(self, requested_path)
    }

    fn remove_dir(&mut self, requested_path: String) -> Result<(), DfsFrontendError> {
        create_remove_dir::remove_dir(self, requested_path)
    }

    fn read(&mut self, file: &FileHandle, count: usize) -> Result<Vec<u8>, DfsFrontendError> {
        if let Some(opened_file) = self.open_files.get_mut(&file.descriptor_uuid) {
            opened_file.read(count)
        } else {
            Err(DfsFrontendError::FileAlreadyClosed)
        }
    }

    fn write(&mut self, file: &FileHandle, buf: Vec<u8>) -> Result<usize, DfsFrontendError> {
        if let Some(opened_file) = self.open_files.get_mut(&file.descriptor_uuid) {
            opened_file.write(&buf)
        } else {
            Err(DfsFrontendError::FileAlreadyClosed)
        }
    }

    fn seek_from_start(
        &mut self,
        file: &FileHandle,
        pos_from_start: usize,
    ) -> Result<usize, DfsFrontendError> {
        self.seek(
            file,
            SeekFrom::Start {
                position: pos_from_start,
            },
        )
    }

    fn seek_from_current(
        &mut self,
        file: &FileHandle,
        pos_from_current: isize,
    ) -> Result<usize, DfsFrontendError> {
        self.seek(
            file,
            SeekFrom::Current {
                offset: pos_from_current,
            },
        )
    }

    fn seek_from_end(
        &mut self,
        file: &FileHandle,
        pos_from_end: usize,
    ) -> Result<usize, DfsFrontendError> {
        self.seek(
            file,
            SeekFrom::End {
                remaining: pos_from_end,
            },
        )
    }
}
