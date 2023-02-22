use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;

use wildland_corex::dfs::interface::{NodeType, Stat, UnixTimestamp, WlPermissions};

use super::client::S3Client;
use super::descriptor::S3Descriptor;
use super::file_system::{Directory, FileSystemNodeRef};
use super::helpers::{commit_file_system, defuse, load_file_system};
use crate::storage_backends::models::{
    CreateDirResponse,
    CreateFileResponse,
    MetadataResponse,
    OpenResponse,
    ReadDirResponse,
    RemoveDirResponse,
    RemoveFileResponse,
    RenameResponse,
    SetPermissionsResponse,
    StatFsResponse,
    StorageBackendError,
};
use crate::storage_backends::s3::file_system::File;
use crate::storage_backends::StorageBackend;

pub struct S3Backend {
    client: Rc<dyn S3Client>,
    bucket_name: String,
}

impl S3Backend {
    pub fn new(client: Rc<dyn S3Client>, bucket_name: String) -> Self {
        Self {
            client,
            bucket_name,
        }
    }
}

impl StorageBackend for S3Backend {
    fn read_dir(&self, path: &Path) -> Result<ReadDirResponse, StorageBackendError> {
        match load_file_system(&*self.client, &self.bucket_name)?.get_node(path) {
            Some(FileSystemNodeRef::Directory(dir)) => Ok(ReadDirResponse::Entries(
                dir.children.iter().map(|node| node.name().into()).collect(),
            )),
            Some(FileSystemNodeRef::File(_)) => Ok(ReadDirResponse::NotADirectory),
            None => Ok(ReadDirResponse::NoSuchPath),
        }
    }

    fn metadata(&self, path: &Path) -> Result<MetadataResponse, StorageBackendError> {
        match load_file_system(&*self.client, &self.bucket_name)?.get_node(path) {
            Some(FileSystemNodeRef::File(file)) => Ok(MetadataResponse::Found(Stat {
                node_type: NodeType::File,
                size: file.size,
                access_time: None,
                modification_time: Some(file.modification_time.clone()),
                change_time: None,
                permissions: WlPermissions::read_write(), // TODO COR-87 store permission in metadata and retrieve it
            })),
            Some(FileSystemNodeRef::Directory(dir)) => Ok(MetadataResponse::Found(Stat {
                node_type: NodeType::Dir,
                size: 0,
                access_time: None,
                modification_time: Some(dir.modification_time.clone()),
                change_time: None,
                permissions: WlPermissions::read_write(), // TODO COR-87 store permission in metadata and retrieve it
            })),
            None => Ok(MetadataResponse::NotFound),
        }
    }

    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError> {
        match load_file_system(&*self.client, &self.bucket_name)?.get_node(path) {
            Some(FileSystemNodeRef::File(file)) => Ok(OpenResponse::found(S3Descriptor::new(
                self.bucket_name.clone(),
                file.object_name.clone(),
                path.to_owned(),
                file.size,
                file.e_tag.clone(),
                self.client.clone(),
            ))),
            Some(FileSystemNodeRef::Directory(_)) => Ok(OpenResponse::NotAFile),
            None => Ok(OpenResponse::NotFound),
        }
    }

    fn create_dir(&self, path: &Path) -> Result<CreateDirResponse, StorageBackendError> {
        let parent = match path.parent() {
            Some(parent) => parent,
            None => return Ok(CreateDirResponse::InvalidParent),
        };

        let mut file_system = load_file_system(&*self.client, &self.bucket_name)?;

        if file_system.get_node(path).is_some() {
            return Ok(CreateDirResponse::PathAlreadyExists);
        };

        match file_system.get_node(parent) {
            Some(FileSystemNodeRef::Directory(dir)) => dir.children.push(
                Directory {
                    name: path.file_name().unwrap().to_string_lossy().to_string(),
                    children: Vec::new(),
                    modification_time: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|duration| UnixTimestamp {
                            sec: duration.as_secs(),
                            nano_sec: duration.subsec_nanos(),
                        })
                        .unwrap(),
                }
                .into(),
            ),
            Some(FileSystemNodeRef::File(_)) => return Ok(CreateDirResponse::InvalidParent),
            None => return Ok(CreateDirResponse::InvalidParent),
        };

        commit_file_system(&*self.client, &self.bucket_name, file_system)?;
        Ok(CreateDirResponse::Created)
    }

    fn remove_dir(&self, path: &Path) -> Result<RemoveDirResponse, StorageBackendError> {
        let parent = match path.parent() {
            Some(parent) => parent,
            None => return Ok(RemoveDirResponse::RootRemovalNotAllowed),
        };

        let mut file_system = load_file_system(&*self.client, &self.bucket_name)?;

        match file_system.get_node(path) {
            Some(FileSystemNodeRef::Directory(dir)) if dir.children.is_empty() => (),
            Some(FileSystemNodeRef::Directory(_)) => return Ok(RemoveDirResponse::DirNotEmpty),
            Some(FileSystemNodeRef::File(_)) => return Ok(RemoveDirResponse::NotADirectory),
            None => return Ok(RemoveDirResponse::NotFound),
        };

        match file_system.get_node(parent) {
            Some(FileSystemNodeRef::Directory(dir)) => dir
                .children
                .retain(|node| node.name() != path.file_name().unwrap()),
            Some(FileSystemNodeRef::File(_)) => return Ok(RemoveDirResponse::NotFound),
            None => return Ok(RemoveDirResponse::NotFound),
        }

        commit_file_system(&*self.client, &self.bucket_name, file_system)?;
        Ok(RemoveDirResponse::Removed)
    }

    fn path_exists(&self, path: &Path) -> Result<bool, StorageBackendError> {
        Ok(load_file_system(&*self.client, &self.bucket_name)?
            .get_node(path)
            .is_some())
    }

    fn remove_file(&self, path: &Path) -> Result<RemoveFileResponse, StorageBackendError> {
        let parent = match path.parent() {
            Some(parent) => parent,
            None => return Ok(RemoveFileResponse::NotFound),
        };

        let mut file_system = load_file_system(&*self.client, &self.bucket_name)?;

        let object_name_to_remove = match file_system.get_node(path) {
            Some(FileSystemNodeRef::File(file)) => file.object_name.clone(),
            Some(FileSystemNodeRef::Directory(_)) => return Ok(RemoveFileResponse::NotAFile),
            None => return Ok(RemoveFileResponse::NotFound),
        };

        match file_system.get_node(parent) {
            Some(FileSystemNodeRef::Directory(dir)) => dir
                .children
                .retain(|node| node.name() != path.file_name().unwrap()),
            Some(FileSystemNodeRef::File(_)) => return Ok(RemoveFileResponse::NotFound),
            None => return Ok(RemoveFileResponse::NotFound),
        }

        commit_file_system(&*self.client, &self.bucket_name, file_system)?;

        let _ = self
            .client
            .remove_object(&object_name_to_remove, &self.bucket_name);

        Ok(RemoveFileResponse::Removed)
    }

    fn create_file(&self, path: &Path) -> Result<CreateFileResponse, StorageBackendError> {
        let parent = match path.parent() {
            Some(parent) => parent,
            None => return Ok(CreateFileResponse::InvalidParent),
        };

        let mut file_system = load_file_system(&*self.client, &self.bucket_name)?;
        let old_file = match file_system.get_node(path) {
            Some(FileSystemNodeRef::File(file)) => Some(file.clone()),
            Some(FileSystemNodeRef::Directory(_)) => return Ok(CreateFileResponse::PathTakenByDir),
            None => None,
        };

        let parent_dir = match file_system.get_node(parent) {
            Some(FileSystemNodeRef::Directory(dir)) => dir,
            Some(FileSystemNodeRef::File(_)) => return Ok(CreateFileResponse::InvalidParent),
            None => return Ok(CreateFileResponse::InvalidParent),
        };

        if let Some(old_file) = &old_file {
            parent_dir
                .children
                .retain(|node| node.name() != old_file.name)
        }

        let new_file = self
            .client
            .create_new_empty(&self.bucket_name)
            .map_err(|err| StorageBackendError::Generic(err.into()))?;

        let remove_new_file_on_exit = scopeguard::guard((), |_| {
            tracing::error!("Failed to create new file. Aborting.");
            if self
                .client
                .remove_object(&new_file.object_name, &self.bucket_name)
                .is_err()
            {
                tracing::error!("Failed to remove new file");
            }
        });

        parent_dir.children.push(
            File {
                name: path.file_name().unwrap().to_string_lossy().to_string(),
                object_name: new_file.object_name.clone(),
                size: 0,
                e_tag: new_file.e_tag.clone(),
                modification_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|duration| UnixTimestamp {
                        sec: duration.as_secs(),
                        nano_sec: duration.subsec_nanos(),
                    })
                    .unwrap(),
            }
            .into(),
        );

        commit_file_system(&*self.client, &self.bucket_name, file_system)?;

        defuse(remove_new_file_on_exit);

        if let Some(old_file) = old_file {
            let _ = self
                .client
                .remove_object(&old_file.object_name, &self.bucket_name);
        };

        Ok(CreateFileResponse::created(S3Descriptor::new(
            self.bucket_name.clone(),
            new_file.object_name,
            path.to_owned(),
            0,
            new_file.e_tag,
            self.client.clone(),
        )))
    }

    fn rename(
        &self,
        _old_path: &Path,
        _new_path: &Path,
    ) -> Result<RenameResponse, StorageBackendError> {
        todo!() // TODO COR-87
    }

    fn set_permissions(
        &self,
        _path: &Path,
        _permissions: WlPermissions,
    ) -> Result<SetPermissionsResponse, StorageBackendError> {
        todo!() // TODO COR-87
    }

    fn stat_fs(&self, _path: &Path) -> Result<StatFsResponse, StorageBackendError> {
        todo!() // TODO COR-87
    }
}
