use rustbreak::error::RustbreakError;
use thiserror::Error;

pub type CatlibResult<T> = Result<T, CatlibError>;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CatlibError {
    #[error("No records found")]
    NoRecordsFound,
    #[error("Corrupted database records")]
    MalformedDatabaseEntry,
    #[error("Entry already exists")]
    RecordAlreadyExists,
    #[error("Catlib error: {0}")]
    Generic(String),
}

impl From<RustbreakError> for CatlibError {
    fn from(rb_error: RustbreakError) -> Self {
        match rb_error {
            RustbreakError::DeSerialization(_) => {
                CatlibError::Generic("RustbreakError::DeSerialization".into())
            }
            RustbreakError::Poison => CatlibError::Generic("RustbreakError::Poison".into()),
            RustbreakError::Backend(_) => CatlibError::Generic("RustbreakError::Backend".into()),
            RustbreakError::WritePanic => CatlibError::Generic("RustbreakError::WritePanic".into()),
            _ => CatlibError::Generic("Unknown Rustbreak error".into()),
        }
    }
}
