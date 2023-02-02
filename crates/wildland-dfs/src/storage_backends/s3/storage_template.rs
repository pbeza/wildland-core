use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct S3StorageTemplate {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub bucket_name: String,
}
