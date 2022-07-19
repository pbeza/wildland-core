use crate::error::LSSError;

mod api;
mod error;
mod file;

pub type LSSResult<T> = Result<T, LSSError>;
