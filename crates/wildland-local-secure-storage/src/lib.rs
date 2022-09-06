mod api;
mod file;

pub use api::LocalSecureStorage;
pub use file::FileLSS;
use rustbreak::RustbreakError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[error("Local Secure Storage error: {0}")]
pub struct LssError(pub String);

impl From<RustbreakError> for LssError {
    fn from(e: RustbreakError) -> Self {
        Self(e.to_string())
    }
}

pub type LssResult<T> = Result<T, LssError>;
