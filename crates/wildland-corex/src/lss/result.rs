use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[error("Local Secure Storage error: {0}")]
pub struct LssError(pub String);

pub type LssResult<T> = Result<T, LssError>;
