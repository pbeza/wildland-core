use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;

use wildland_corex::dfs::interface::{NodeType, Stat, UnixTimestamp};

use super::client::S3Client;
use super::descriptor::S3Descriptor;
use super::file_system::{Directory, Node};
use super::helpers::{commit_file_system, load_file_system};
use crate::storage_backends::models::{
    CreateDirResponse,
    CreateFileResponse,
    MetadataResponse,
    OpenResponse,
    ReadDirResponse,
    RemoveDirResponse,
    RemoveFileResponse,
    RenameResponse,
    StorageBackendError,
};
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
            Some(Node::Directory(dir)) => Ok(ReadDirResponse::Entries(
                dir.children.iter().map(|node| node.name().into()).collect(),
            )),
            Some(Node::File(_)) => Ok(ReadDirResponse::NotADirectory),
            None => Ok(ReadDirResponse::NoSuchPath),
        }
    }

    fn metadata(&self, path: &Path) -> Result<MetadataResponse, StorageBackendError> {
        match load_file_system(&*self.client, &self.bucket_name)?.get_node(path) {
            Some(Node::File(file)) => Ok(MetadataResponse::Found(Stat {
                node_type: NodeType::File,
                size: file.size,
                access_time: None,
                modification_time: Some(file.modification_time.clone()),
                change_time: None,
            })),
            Some(Node::Directory(dir)) => Ok(MetadataResponse::Found(Stat {
                node_type: NodeType::Dir,
                size: 0,
                access_time: None,
                modification_time: Some(dir.modification_time.clone()),
                change_time: None,
            })),
            None => Ok(MetadataResponse::NotFound),
        }
    }

    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError> {
        match load_file_system(&*self.client, &self.bucket_name)?.get_node(path) {
            Some(Node::File(file)) => Ok(OpenResponse::found(S3Descriptor::new(
                self.bucket_name.clone(),
                file.object_name.clone(),
                path.to_owned(),
                file.size,
                file.e_tag.clone(),
                self.client.clone(),
            ))),
            Some(Node::Directory(_)) => Ok(OpenResponse::NotAFile),
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
            Some(Node::Directory(dir)) => dir.children.push(
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
            Some(Node::File(_)) => return Ok(CreateDirResponse::InvalidParent),
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
            Some(Node::Directory(dir)) if dir.children.is_empty() => (),
            Some(Node::Directory(_)) => return Ok(RemoveDirResponse::DirNotEmpty),
            Some(Node::File(_)) => return Ok(RemoveDirResponse::NotADirectory),
            None => return Ok(RemoveDirResponse::NotFound),
        };

        match file_system.get_node(parent) {
            Some(Node::Directory(dir)) => dir
                .children
                .retain(|node| node.name() != path.file_name().unwrap()),
            Some(Node::File(_)) => return Ok(RemoveDirResponse::NotFound),
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
            Some(Node::File(file)) => file.object_name.clone(),
            Some(Node::Directory(_)) => return Ok(RemoveFileResponse::NotAFile),
            None => return Ok(RemoveFileResponse::NotFound),
        };

        match file_system.get_node(parent) {
            Some(Node::Directory(dir)) => dir
                .children
                .retain(|node| node.name() != path.file_name().unwrap()),
            Some(Node::File(_)) => return Ok(RemoveFileResponse::NotFound),
            None => return Ok(RemoveFileResponse::NotFound),
        }

        commit_file_system(&*self.client, &self.bucket_name, file_system)?;

        let _ = self
            .client
            .remove_object(&object_name_to_remove, &self.bucket_name);

        Ok(RemoveFileResponse::Removed)
    }

    fn create_file(&self, _path: &Path) -> Result<CreateFileResponse, StorageBackendError> {
        todo!() // TODO COR-87
    }

    fn rename(
        &self,
        _old_path: &Path,
        _new_path: &Path,
    ) -> Result<RenameResponse, StorageBackendError> {
        todo!() // TODO COR-87
    }
}
