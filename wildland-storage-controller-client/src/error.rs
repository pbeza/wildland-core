use common::error::CorexCommonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CorexSCClientError {
    #[error("{0}")]
    HttpError(String),
    #[error("Cannot serialize request")]
    CannotSerializeRequestError { source: serde_json::Error },
    #[error(transparent)]
    CommonLibError(#[from] CorexCommonError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
