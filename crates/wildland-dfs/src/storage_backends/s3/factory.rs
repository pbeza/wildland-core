use std::rc::Rc;

use anyhow::Context;
use aws_sdk_s3::{Credentials, Region};
use tokio::runtime::Runtime;
use wildland_corex::Storage;

use super::backend::S3Backend;
use super::client::WildlandS3Client;
use super::storage_template::S3StorageTemplate;
use crate::storage_backends::{StorageBackend, StorageBackendFactory};

pub struct S3BackendFactory {
    rt: Rc<Runtime>,
}

impl S3BackendFactory {
    pub fn new() -> Self {
        Self {
            rt: Rc::new(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            ),
        }
    }
}

impl Default for S3BackendFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackendFactory for S3BackendFactory {
    fn init_backend(&self, storage: Storage) -> anyhow::Result<Rc<dyn StorageBackend>> {
        let S3StorageTemplate {
            access_key_id,
            secret_access_key,
            region,
            bucket_name,
            endpoint_url,
        } = serde_json::from_value(storage.data()).context("Invalid S3 storage template")?;

        let credentials = Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "WildlandS3Client",
        );

        let region = Region::new(region);
        let client = Rc::new(WildlandS3Client::new(
            self.rt.clone(),
            credentials,
            region,
            endpoint_url,
        ));

        Ok(Rc::new(S3Backend::new(client, bucket_name)))
    }
}
