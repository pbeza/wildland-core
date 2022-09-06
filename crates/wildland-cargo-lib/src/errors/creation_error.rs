use super::{ErrDomain, ExceptionTrait, WildlandXDomain};
use std::fmt::Display;

pub type CreationResult<T, E> = Result<T, CreationError<E>>;
#[derive(Debug, Clone)]
#[repr(C)]
pub enum CreationError<T: Clone> {
    NotCreated(T),
}

impl<E: Display + Clone + ErrDomain> ExceptionTrait for CreationError<E> {
    fn reason(&self) -> String {
        match self {
            Self::NotCreated(e) => e.to_string(),
        }
    }

    fn domain(&self) -> WildlandXDomain {
        match self {
            CreationError::NotCreated(e) => e.domain(),
        }
    }
}
