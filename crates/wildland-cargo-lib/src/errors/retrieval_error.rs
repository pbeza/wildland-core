use super::ExceptionTrait;
use std::fmt::Display;

pub type RetrievalResult<T, E> = Result<T, RetrievalError<E>>;
#[derive(Debug, Clone)]
#[repr(C)]
pub enum RetrievalError<E: Clone> {
    NotFound(String),
    Unexpected(E),
}

impl<E: Display + Clone> ExceptionTrait for RetrievalError<E> {
    fn reason(&self) -> String {
        match self {
            RetrievalError::NotFound(s) => s.to_string(),
            RetrievalError::Unexpected(e) => e.to_string(),
        }
    }
}
