use std::rc::Rc;
use thiserror::Error;
use wildland_crypto::error::CryptoError;

#[derive(Error, Debug, Clone)]
pub enum WildlandHttpClientError {
    #[error("{0}")]
    HttpError(String),
    #[error("Cannot serialize request")]
    CannotSerializeRequestError { source: Rc<serde_json::Error> },
    #[error(transparent)]
    CommonLibError(#[from] CryptoError),
    #[error(transparent)]
    HttpLibError(#[from] Rc<minreq::Error>),
    #[error("No body in the response")]
    NoBody,
}
