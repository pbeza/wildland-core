use thiserror::Error;
use wildland_crypto::error::CryptoError;

#[derive(Error, Debug)]
pub enum WildlandHttpClientError {
    #[error("{0}")]
    HttpError(String),
    #[error("Cannot serialize request")]
    CannotSerializeRequestError { source: serde_json::Error },
    #[error(transparent)]
    CommonLibError(#[from] CryptoError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
