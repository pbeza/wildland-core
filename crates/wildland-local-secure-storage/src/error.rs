use rustbreak::RustbreakError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LSSError {
    #[error(transparent)]
    FileLSSError(#[from] RustbreakError),
}
