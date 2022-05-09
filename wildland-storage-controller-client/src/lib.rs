use serde::{Deserialize, Serialize};

pub mod client;
mod constants;
pub mod credentials;
pub mod error;
pub mod metrics;
pub(crate) mod response_handler;
pub mod signature;
pub mod storage;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCredentialsRes {
    #[serde(rename(deserialize = "credentialID"))]
    pub credentials_id: String,
    #[serde(rename(deserialize = "credentialSecret"))]
    pub credentials_secret: String,
}
