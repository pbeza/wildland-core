use std::path::Path;
use std::rc::Rc;

use super::client::S3Client;
use super::descriptor::S3Descriptor;
use super::error::S3Error;
use crate::storage_backends::models::{
    GetattrResponse,
    OpenResponse,
    ReaddirResponse,
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
    fn readdir(&self, path_within_storage: &Path) -> Result<ReaddirResponse, StorageBackendError> {
        match self
            .client
            .list_files(path_within_storage, &self.bucket_name)
        {
            // TODO COR-86 handle NoSuchPath and NotADirectory in different way
            Ok(vec) => Ok(ReaddirResponse::Entries(vec)),
            Err(S3Error::NotFound) => Ok(ReaddirResponse::NotADirectory),
            Err(err @ S3Error::ETagMistmach) => Err(StorageBackendError::Generic(err.into())),
            Err(err @ S3Error::NoSuchBucket) => Err(StorageBackendError::Generic(err.into())),
            Err(err @ S3Error::Generic(_)) => Err(StorageBackendError::Generic(err.into())),
        }
    }

    fn getattr(&self, path_within_storage: &Path) -> Result<GetattrResponse, StorageBackendError> {
        match self
            .client
            .get_object_attributes(path_within_storage, &self.bucket_name)
        {
            Ok(v) => Ok(GetattrResponse::Found(v.stat)),
            Err(S3Error::NotFound) => Ok(GetattrResponse::NotFound),
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
}
