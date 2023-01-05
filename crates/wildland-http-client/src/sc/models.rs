use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStorageRes {
    #[serde(rename(deserialize = "id"))]
    pub storage_id: String,
    #[serde(rename(deserialize = "credentialID"))]
    pub credentials_id: String,
    #[serde(rename(deserialize = "credentialSecret"))]
    pub credentials_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureRequestReq {
    #[serde(rename(serialize = "credentialID"))]
    pub credential_id: String,
    pub timestamp: String,
    #[serde(rename(serialize = "storageMethod"))]
    pub storage_method: String,
    #[serde(rename(serialize = "storagePath"))]
    pub storage_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureRequestRes {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCredentialsReq {
    #[serde(rename(serialize = "credentialID"))]
    pub credential_id: String,
    pub timestamp: String,
    #[serde(rename(serialize = "credPermission"))]
    pub cred_permission: String,
    #[serde(rename(serialize = "fsPermission"))]
    pub fs_permission: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCredentialsRes {
    #[serde(rename(deserialize = "credentialID"))]
    pub credentials_id: String,
    #[serde(rename(deserialize = "credentialSecret"))]
    pub credentials_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMetricsReq {
    #[serde(rename(serialize = "credentialID"))]
    pub credential_id: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMetricsRes {
    #[serde(rename(deserialize = "credentialID"))]
    pub credentials_id: String,
    #[serde(rename(deserialize = "usageCred"))]
    pub usage_cred: UsageReq,
    #[serde(rename(deserialize = "usageStorage"))]
    pub usage_storage: UsageReq,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageReq {
    pub rx: i64,
    pub tx: i64,
}
