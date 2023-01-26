use std::path::{Path, PathBuf};

use wildland_corex::dfs::interface::Stat;

use super::client::S3Client;
use crate::storage_backends::{StorageBackend, StorageBackendError};

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
    fn readdir(&self, path_within_storage: &Path) -> Result<Vec<PathBuf>, StorageBackendError> {
        Ok(self
            .client
            .list_files(path_within_storage, &self.bucket_name)?)
    }

    fn getattr(&self, path_within_storage: &Path) -> Result<Stat, StorageBackendError> {
        Ok(self
            .client
            .get_object_attributes(path_within_storage, &self.bucket_name)?)
    }
}
