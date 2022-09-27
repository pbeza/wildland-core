use std::sync::Arc;
use thiserror::Error;
use wildland_crypto::error::CryptoError;

#[derive(Error, Debug, Clone)]
pub enum WildlandHttpClientError {
    #[error("{0}")]
    HttpError(String),
    #[error("Cannot serialize request")]
    CannotSerializeRequestError { source: Arc<serde_json::Error> },
    #[error(transparent)]
    CommonLibError(#[from] CryptoError),
    #[error(transparent)]
    ReqwestError(#[from] Arc<reqwest::Error>),
}
