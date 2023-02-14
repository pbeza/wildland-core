use std::path::{Component, Path};
use std::rc::Rc;

use wildland_corex::dfs::interface::NodeType;

use super::client::S3Client;
use super::descriptor::S3Descriptor;
use super::error::S3Error;
use crate::storage_backends::models::{
    CreateDirResponse,
    MetadataResponse,
    OpenResponse,
    ReadDirResponse,
    RemoveDirResponse,
    RemoveFileResponse,
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
        match self.client.list_files(path, &self.bucket_name) {
            // TODO COR-86 handle NoSuchPath and NotADirectory in different way
            Ok(vec) => Ok(ReadDirResponse::Entries(vec)),
            Err(S3Error::NotFound) => Ok(ReadDirResponse::NotADirectory),
            Err(err @ (S3Error::ETagMistmach | S3Error::Generic(_))) => {
                Err(StorageBackendError::Generic(err.into()))
            }
        }
    }

    fn metadata(&self, path: &Path) -> Result<MetadataResponse, StorageBackendError> {
        match self.client.get_object_attributes(path, &self.bucket_name) {
            Ok(v) => Ok(MetadataResponse::Found(v.stat)),
            Err(S3Error::NotFound) => Ok(MetadataResponse::NotFound),
            Err(err @ (S3Error::ETagMistmach | S3Error::Generic(_))) => {
                Err(StorageBackendError::Generic(err.into()))
            }
        }
    }

    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError> {
        match self.client.get_object_attributes(path, &self.bucket_name) {
            Ok(attrs) if attrs.stat.node_type == NodeType::File => {
                Ok(OpenResponse::found(S3Descriptor::new(
                    self.bucket_name.clone(),
                    path.into(),
                    attrs.stat.size,
                    attrs.etag,
                    self.client.clone(),
                )))
            }
            Ok(_) => Ok(OpenResponse::NotAFile),
            Err(S3Error::NotFound) => Ok(OpenResponse::NotFound),
            Err(err @ (S3Error::ETagMistmach | S3Error::Generic(_))) => {
                Err(StorageBackendError::Generic(err.into()))
            }
        }
    }

    fn create_dir(&self, path: &Path) -> Result<CreateDirResponse, StorageBackendError> {
        let components: Vec<_> = path.components().collect();

        match components.as_slice() {
            // parent dir is root. no need to check if it exists.
            [Component::RootDir, _] => (),
            _ => {
                match self
                    .client
                    .get_object_attributes(path.parent().unwrap(), &self.bucket_name)
                {
                    Ok(attrs) if attrs.stat.node_type == NodeType::Dir => (),
                    Ok(_) => return Ok(CreateDirResponse::InvalidParent),
                    Err(S3Error::NotFound) => return Ok(CreateDirResponse::InvalidParent),
                    Err(err @ (S3Error::ETagMistmach | S3Error::Generic(_))) => {
                        return Err(StorageBackendError::Generic(err.into()))
                    }
                }
            }
        };

        match self.client.get_object_attributes(path, &self.bucket_name) {
            Err(S3Error::NotFound) => (),
            Ok(_) => return Ok(CreateDirResponse::PathAlreadyExists),
            Err(err @ (S3Error::ETagMistmach | S3Error::Generic(_))) => {
                return Err(StorageBackendError::Generic(err.into()))
            }
        };

        match self.client.create_dir(path, &self.bucket_name) {
            Ok(_) => Ok(CreateDirResponse::Created),
            Err(err @ (S3Error::NotFound | S3Error::ETagMistmach | S3Error::Generic(_))) => {
                Err(StorageBackendError::Generic(err.into()))
            }
        }
    }

    fn remove_dir(&self, path: &Path) -> Result<RemoveDirResponse, StorageBackendError> {
        if path == Path::new("/") {
            return Ok(RemoveDirResponse::RootRemovalNotAllowed);
        }

        match self.read_dir(path)? {
            ReadDirResponse::Entries(children) if children.is_empty() => {
                match self.client.remove_object(path, &self.bucket_name) {
                    Ok(_) => Ok(RemoveDirResponse::Removed),
                    Err(
                        err @ (S3Error::NotFound | S3Error::ETagMistmach | S3Error::Generic(_)),
                    ) => Err(StorageBackendError::Generic(err.into())),
                }
            }
            ReadDirResponse::Entries(_) => Ok(RemoveDirResponse::DirNotEmpty),
            ReadDirResponse::NoSuchPath => Ok(RemoveDirResponse::NotFound),
            ReadDirResponse::NotADirectory => Ok(RemoveDirResponse::NotADirectory),
        }
    }

    fn path_exists(&self, _path: &Path) -> Result<bool, StorageBackendError> {
        todo!() // TODO COR-87
    }

    fn remove_file(&self, _path: &Path) -> Result<RemoveFileResponse, StorageBackendError> {
        todo!() // TODO COR-87
    }
}
