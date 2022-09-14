use super::{ErrDomain, ExceptionTrait, WildlandXDomain};
use std::fmt::Display;

pub type RetrievalResult<T, E> = Result<T, RetrievalError<E>>;
#[derive(Debug, Clone)]
#[repr(C)]
pub enum RetrievalError<E: Clone> {
    NotFound(String),
    Unexpected(E),
}

impl<E: Display + Clone + ErrDomain> ExceptionTrait for RetrievalError<E> {
    fn reason(&self) -> String {
        match self {
            RetrievalError::NotFound(s) => s.to_string(),
            RetrievalError::Unexpected(e) => e.to_string(),
        }
    }

    fn domain(&self) -> WildlandXDomain {
        match self {
            RetrievalError::NotFound(_) => WildlandXDomain::CargoUser,
            RetrievalError::Unexpected(e) => e.domain(),
        }
    }
}
