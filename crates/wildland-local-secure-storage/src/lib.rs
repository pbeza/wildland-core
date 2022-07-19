use crate::error::LSSError;

mod api;
mod file;
mod error;

pub type LSSResult<T> = Result<T, LSSError>;