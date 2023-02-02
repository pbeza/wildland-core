use std::path::Path;

use super::client::S3Client;
use crate::storage_backends::{
    GetattrResponse, OpenResponse, ReaddirResponse, StorageBackend, StorageBackendError,
};

pub struct S3Backend {
    client: Box<dyn S3Client>,
    bucket_name: String,
}

impl S3Backend {
    pub fn new(client: Box<dyn S3Client>, bucket_name: String) -> Self {
        Self {
            client,
            bucket_name,
        }
    }
}

impl StorageBackend for S3Backend {
    fn readdir(&self, path_within_storage: &Path) -> Result<ReaddirResponse, StorageBackendError> {
        Ok(ReaddirResponse::Entries(
            self.client
                .list_files(path_within_storage, &self.bucket_name)?,
        ))
    }

    fn getattr(&self, path_within_storage: &Path) -> Result<GetattrResponse, StorageBackendError> {
        Ok(GetattrResponse::Found(self.client.get_object_attributes(
            path_within_storage,
            &self.bucket_name,
        )?))
    }

    fn open(&self, _path: &Path) -> Result<OpenResponse, StorageBackendError> {
        todo!()
    }

    fn create_dir(
        &self,
        path: &Path,
    ) -> Result<crate::storage_backends::CreateDirResponse, StorageBackendError> {
        todo!()
    }
}
