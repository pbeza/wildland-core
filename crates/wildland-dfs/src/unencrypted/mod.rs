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

mod create_remove;
mod metadata;
mod node_descriptor;
mod path_translator;
mod read_dir;
#[cfg(test)]
mod tests;
mod utils;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use uuid::Uuid;
use wildland_corex::dfs::interface::{
    DfsFrontend, DfsFrontendError, FileHandle, FsStat, Stat, UnixTimestamp, WlPermissions,
};
use wildland_corex::{PathResolver, Storage};

use self::node_descriptor::NodeDescriptor;
use self::path_translator::uuid_in_dir::UuidInDirTranslator;
use self::path_translator::PathConflictResolver;
use self::utils::{
    exec_on_opened_file, exec_on_single_existing_node, execute_container_operation,
    filter_existent_nodes, get_related_nodes,
};
use crate::storage_backends::models::{
    CloseError, OpenResponse, RenameResponse, SeekFrom, SetPermissionsResponse, StatFsResponse,
};
use crate::storage_backends::{
    CloseOnDropDescriptor, OpenedFileDescriptor, StorageBackend, StorageBackendFactory,
};
use crate::unencrypted::utils::find_node_matching_requested_path;

// TODO WILX-387 Current DFS implementation uses some kind of mapping paths into another ones in order to
// avoid conflicts. There is always a probability that mapped path will be in conflict with some
// other user's file or directory (now we assume that user won't have any file named as an uuid
// string format). The problem could be solved by checking mapped path (e.g. containing uuid) if
// it represents a file/dir in the first place, and then, if no results are found, to check if
// the conflict resolution took place and find files which paths were mapped.

pub struct UnencryptedDfs {
    opened_files: HashMap<Uuid, CloseOnDropDescriptor>,

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
            opened_files: Default::default(),
            path_resolver,
            storage_backend_factories,
            storage_backends: HashMap::new(),
            path_translator: Box::new(UuidInDirTranslator::new()),
        }
    }

    fn insert_opened_file(&mut self, opened_file: CloseOnDropDescriptor) -> FileHandle {
        let uuid = Uuid::new_v4();
        self.opened_files.insert(uuid, opened_file);
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
        exec_on_opened_file(self, file, &|opened_file| opened_file.seek(seek_from))
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
    fn read_dir(&mut self, requested_path: String) -> Result<Vec<String>, DfsFrontendError> {
        read_dir::read_dir(self, requested_path)
    }

    // Returns Stat of the file indicated by the provided exposed path
    fn metadata(&mut self, input_exposed_path: String) -> Result<Stat, DfsFrontendError> {
        metadata::metadata(self, input_exposed_path)
    }

    fn open(&mut self, input_exposed_path: String) -> Result<FileHandle, DfsFrontendError> {
        let open_op = |dfs: &mut UnencryptedDfs, node: &NodeDescriptor| match node {
            NodeDescriptor::Physical { storages, .. } => {
                execute_container_operation(dfs, storages, &|backend| {
                    backend.open(storages.path_within_storage())
                })
                .and_then(|resp| match resp {
                    OpenResponse::Found(opened_file) => Ok(dfs.insert_opened_file(opened_file)),
                    OpenResponse::NotAFile => Err(DfsFrontendError::NotAFile),
                    _ => Err(DfsFrontendError::NoSuchPath),
                })
            }
            NodeDescriptor::Virtual { .. } => Err(DfsFrontendError::NoSuchPath),
        };

        let input_exposed_path = Path::new(&input_exposed_path);
        let nodes = get_related_nodes(self, input_exposed_path)?;

        match nodes.as_slice() {
            [] => Err(DfsFrontendError::NoSuchPath),
            [node] => open_op(self, node),
            _ => {
                let existent_paths: Vec<_> = filter_existent_nodes(&nodes, self)?.collect();

                match existent_paths.as_slice() {
                    [] => Err(DfsFrontendError::NoSuchPath),
                    [node] => open_op(self, node),
                    _ => {
                        let exposed_paths = self.path_translator.solve_conflicts(existent_paths);
                        let node =
                            find_node_matching_requested_path(input_exposed_path, &exposed_paths);
                        match node {
                            // Physical node
                            Some(node @ NodeDescriptor::Physical { .. }) => open_op(self, node),
                            // Virtual node
                            Some(_) | None if !exposed_paths.is_empty() => {
                                Err(DfsFrontendError::NotAFile)
                            }
                            _ => Err(DfsFrontendError::NoSuchPath),
                        }
                    }
                }
            }
        }
    }

    fn close(&mut self, file: &FileHandle) -> Result<(), DfsFrontendError> {
        if let Some(opened_file) = self.opened_files.remove(&file.descriptor_uuid) {
            opened_file.close().map_err(|e| match e {
                CloseError::FileAlreadyClosed => DfsFrontendError::FileAlreadyClosed,
            })
        } else {
            Err(DfsFrontendError::FileAlreadyClosed)
        }
    }

    fn remove_file(&mut self, input_exposed_path: String) -> Result<(), DfsFrontendError> {
        create_remove::remove_file(self, input_exposed_path)
    }

    fn create_file(&mut self, requested_path: String) -> Result<FileHandle, DfsFrontendError> {
        create_remove::create_file(self, requested_path)
    }

    fn create_dir(&mut self, requested_path: String) -> Result<(), DfsFrontendError> {
        create_remove::create_dir(self, requested_path)
    }

    fn remove_dir(&mut self, requested_path: String) -> Result<(), DfsFrontendError> {
        create_remove::remove_dir(self, requested_path)
    }

    fn read(&mut self, file: &FileHandle, count: usize) -> Result<Vec<u8>, DfsFrontendError> {
        exec_on_opened_file(self, file, &|opened_file| opened_file.read(count))
    }

    fn write(&mut self, file: &FileHandle, buf: Vec<u8>) -> Result<usize, DfsFrontendError> {
        exec_on_opened_file(self, file, &|opened_file| opened_file.write(&buf))
    }

    fn seek_from_start(
        &mut self,
        file: &FileHandle,
        pos_from_start: u64,
    ) -> Result<usize, DfsFrontendError> {
        self.seek(
            file,
            SeekFrom::Start {
                offset: pos_from_start,
            },
        )
    }

    fn seek_from_current(
        &mut self,
        file: &FileHandle,
        pos_from_current: i64,
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
        pos_from_end: i64,
    ) -> Result<usize, DfsFrontendError> {
        self.seek(
            file,
            SeekFrom::End {
                offset: pos_from_end,
            },
        )
    }

    fn rename(&mut self, old_path: String, new_path: String) -> Result<(), DfsFrontendError> {
        let old_path = PathBuf::from_str(&old_path).unwrap();
        let mut nodes = get_related_nodes(self, &old_path)?;

        let rename_node = |dfs: &mut UnencryptedDfs, node: &NodeDescriptor| match node {
            node @ NodeDescriptor::Physical { storages, .. } => {
                let abs_path = node.abs_path().to_string_lossy().to_string();
                let claimed_container_root_path = abs_path
                    .strip_suffix(&storages.path_within_storage().to_string_lossy().to_string())
                    .unwrap_or("");

                if let Some(new_path_within_storage) =
                    new_path.strip_prefix(claimed_container_root_path)
                {
                    execute_container_operation(dfs, storages, &|backend| {
                        backend.rename(
                            storages.path_within_storage(),
                            Path::new(new_path_within_storage),
                        )
                    })
                    .and_then(|response| match response {
                        RenameResponse::Renamed => Ok(()),
                        RenameResponse::NotFound => Err(DfsFrontendError::NoSuchPath),
                        RenameResponse::SourceIsParentOfTarget => {
                            Err(DfsFrontendError::SourceIsParentOfTarget)
                        }
                        RenameResponse::TargetPathAlreadyExists => {
                            Err(DfsFrontendError::PathAlreadyExists)
                        }
                    })
                } else {
                    Err(DfsFrontendError::MoveBetweenContainers)
                }
            }
            NodeDescriptor::Virtual { .. } => Err(DfsFrontendError::ReadOnlyPath),
        };

        exec_on_single_existing_node(self, &mut nodes, &rename_node)
    }

    fn set_permissions(
        &mut self,
        input_exposed_path: String,
        permissions: WlPermissions,
    ) -> Result<(), DfsFrontendError> {
        let input_exposed_path = Path::new(&input_exposed_path);

        let set_permissions_op = |dfs: &mut UnencryptedDfs, node: &NodeDescriptor| match node {
            NodeDescriptor::Physical { storages, .. } => {
                execute_container_operation(dfs, storages, &|backend| {
                    backend.set_permissions(storages.path_within_storage(), permissions.clone())
                })
                .and_then(|resp| match resp {
                    SetPermissionsResponse::Set => Ok(()),
                    SetPermissionsResponse::NotFound => Err(DfsFrontendError::NoSuchPath),
                })
            }
            NodeDescriptor::Virtual { .. } => Err(DfsFrontendError::ReadOnlyPath),
        };

        let mut nodes = get_related_nodes(self, input_exposed_path)?;

        exec_on_single_existing_node(self, &mut nodes, &set_permissions_op)
    }

    fn set_owner(&mut self, _path: String) -> Result<(), DfsFrontendError> {
        Err(DfsFrontendError::Generic(
            "`set_owner` is not supported yet".into(),
        ))
    }

    fn set_length(&mut self, file: &FileHandle, length: usize) -> Result<(), DfsFrontendError> {
        exec_on_opened_file(self, file, &|opened_file| opened_file.set_length(length))
    }

    fn sync(&mut self, file: &FileHandle) -> Result<(), DfsFrontendError> {
        exec_on_opened_file(self, file, &|opened_file| opened_file.sync())
    }

    fn set_times(
        &mut self,
        file: &FileHandle,
        access_time: Option<UnixTimestamp>,
        modification_time: Option<UnixTimestamp>,
    ) -> Result<(), DfsFrontendError> {
        exec_on_opened_file(self, file, &|opened_file| {
            opened_file.set_times(access_time.clone(), modification_time.clone())
        })
    }

    fn file_metadata(&mut self, file: &FileHandle) -> Result<Stat, DfsFrontendError> {
        exec_on_opened_file(self, file, &|opened_file| opened_file.metadata())
    }

    fn sync_all(&mut self) -> Result<(), DfsFrontendError> {
        self.opened_files
            .iter_mut()
            .try_for_each(|(_uuid, opened_file)| opened_file.sync())
    }

    fn set_file_permissions(
        &mut self,
        file: &FileHandle,
        permissions: WlPermissions,
    ) -> Result<(), DfsFrontendError> {
        exec_on_opened_file(self, file, &|opened_file| {
            opened_file.set_permissions(permissions.clone())
        })
    }

    fn file_stat_fs(&mut self, file: &FileHandle) -> Result<FsStat, DfsFrontendError> {
        exec_on_opened_file(self, file, &|opened_file| opened_file.stat_fs())
    }

    fn stat_fs(&mut self, input_exposed_path: String) -> Result<FsStat, DfsFrontendError> {
        let input_exposed_path = Path::new(&input_exposed_path);

        let stat_fs_op = |dfs: &mut UnencryptedDfs, node: &NodeDescriptor| match node {
            NodeDescriptor::Physical { storages, .. } => {
                execute_container_operation(dfs, storages, &|backend| {
                    backend.stat_fs(storages.path_within_storage())
                })
                .and_then(|resp| match resp {
                    StatFsResponse::Stat(stats) => Ok(stats),
                    StatFsResponse::NotFound => Err(DfsFrontendError::NoSuchPath),
                    StatFsResponse::NotSupported(msg) => Err(DfsFrontendError::Generic(format!(
                        "`stat_fs` not supported by the backend: {msg}"
                    ))),
                })
            }
            NodeDescriptor::Virtual { .. } => Err(DfsFrontendError::ReadOnlyPath),
        };

        let mut nodes = get_related_nodes(self, input_exposed_path)?;

        exec_on_single_existing_node(self, &mut nodes, &stat_fs_op)
    }
}
