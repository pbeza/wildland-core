use std::path::Path;
use std::rc::Rc;

use super::client::S3Client;
use super::descriptor::S3Descriptor;
use super::error::S3Error;
use crate::storage_backends::models::{
    MetadataResponse,
    OpenResponse,
    ReadDirResponse,
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
    fn read_dir(&self, path_within_storage: &Path) -> Result<ReadDirResponse, StorageBackendError> {
        match self
            .client
            .list_files(path_within_storage, &self.bucket_name)
        {
            // TODO COR-86 handle NoSuchPath and NotADirectory in different way
            Ok(vec) => Ok(ReadDirResponse::Entries(vec)),
            Err(S3Error::NotFound) => Ok(ReadDirResponse::NotADirectory),
            Err(err @ S3Error::ETagMistmach) => Err(StorageBackendError::Generic(err.into())),
            Err(err @ S3Error::NoSuchBucket) => Err(StorageBackendError::Generic(err.into())),
            Err(err @ S3Error::Generic(_)) => Err(StorageBackendError::Generic(err.into())),
        }
    }

    fn metadata(
        &self,
        path_within_storage: &Path,
    ) -> Result<MetadataResponse, StorageBackendError> {
        match self
            .client
            .get_object_attributes(path_within_storage, &self.bucket_name)
        {
            Ok(v) => Ok(MetadataResponse::Found(v.stat)),
            Err(S3Error::NotFound) => Ok(MetadataResponse::NotFound),
            Err(err @ S3Error::ETagMistmach) => Err(StorageBackendError::Generic(err.into())),
            Err(err @ S3Error::NoSuchBucket) => Err(StorageBackendError::Generic(err.into())),
            Err(err @ S3Error::Generic(_)) => Err(StorageBackendError::Generic(err.into())),
        }
    }

    fn open(&self, path: &Path) -> Result<OpenResponse, StorageBackendError> {
        match self.client.get_object_attributes(path, &self.bucket_name) {
            Ok(attrs) => Ok(OpenResponse::found(S3Descriptor::new(
                self.bucket_name.clone(),
                path.into(),
                attrs.stat.size,
                attrs.etag,
                self.client.clone(),
            ))),
            Err(S3Error::NotFound) => Ok(OpenResponse::NotFound),
            Err(err @ S3Error::ETagMistmach) => Err(StorageBackendError::Generic(err.into())),
            Err(err @ S3Error::NoSuchBucket) => Err(StorageBackendError::Generic(err.into())),
            Err(err @ S3Error::Generic(_)) => Err(StorageBackendError::Generic(err.into())),
        }
    }

    fn create_dir(
        &self,
        _path: &Path,
    ) -> Result<crate::storage_backends::CreateDirResponse, StorageBackendError> {
        todo!() // TODO COR-70
    }

    fn remove_dir(
        &self,
        _path: &Path,
    ) -> Result<crate::storage_backends::RemoveDirResponse, StorageBackendError> {
        todo!() // TODO COR-70
    }

    fn path_exists(&self, _path: &Path) -> Result<bool, StorageBackendError> {
        todo!() // TODO COR-87
    }

    fn remove_file(&self, _path: &Path) -> Result<RemoveFileResponse, StorageBackendError> {
        todo!() // TODO COR-87
    }
}
