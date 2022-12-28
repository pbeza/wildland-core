use thiserror::Error;

pub type CatlibResult<T> = Result<T, CatlibError>;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
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
