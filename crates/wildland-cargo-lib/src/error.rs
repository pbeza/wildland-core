use std::fmt::Display;

#[derive(Debug, Clone)]
#[repr(C)]
pub enum WildlandXDomain {
    CoreX,
}

pub trait ExceptionTrait {
    fn reason(&self) -> String;
    fn domain(&self) -> WildlandXDomain;
}

pub type CreationResult<T, E> = Result<T, CreationError<E>>;
#[derive(Debug, Clone)]
#[repr(C)]
pub enum CreationError<T: Clone> {
    NotCreated(T),
}

impl<E: Display + Clone> ExceptionTrait for CreationError<E> {
    fn reason(&self) -> String {
        match self {
            Self::NotCreated(e) => e.to_string(),
        }
    }

    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::CoreX
    }
}

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

    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::CoreX
    }
}
