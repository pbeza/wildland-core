use redis::RedisError;
use thiserror::Error;

pub type CatlibResult<T> = Result<T, CatlibError>;

impl From<RedisError> for CatlibError {
    fn from(err: RedisError) -> Self {
        CatlibError::Generic(format!("Redis error: {err}"))
    }
}

impl From<r2d2::Error> for CatlibError {
    fn from(err: r2d2::Error) -> Self {
        CatlibError::Generic(format!("R2D2 error: {err}"))
    }
}

#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum CatlibError {
    #[error("No records found")]
    NoRecordsFound,
    #[error("Malformed database record")]
    MalformedDatabaseRecord,
    #[error("Record already exists")]
    RecordAlreadyExists,
    #[error("Catlib error: {0}")]
    Generic(String),
}
