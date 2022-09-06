use std::fmt::Display;

use wildland_corex::{CryptoError, ForestRetrievalError};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum WildlandXDomain {
    CargoUser,
    Crypto,
    Catlib,
    CoreX,
    Dfs,
    Lss,
}

pub trait ExceptionTrait {
    fn reason(&self) -> String;
    fn domain(&self) -> WildlandXDomain;
}

pub trait ErrDomain {
    fn domain(&self) -> WildlandXDomain;
}

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

impl ErrDomain for CryptoError {
    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::Crypto
    }
}
impl ErrDomain for wildland_corex::UserCreationError {
    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::CargoUser
    }
}
impl ErrDomain for wildland_corex::LssError {
    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::Lss
    }
}

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

impl ErrDomain for ForestRetrievalError {
    fn domain(&self) -> WildlandXDomain {
        match self {
            ForestRetrievalError::LssError(_) => WildlandXDomain::Lss,
            ForestRetrievalError::KeypairParseError(_) => WildlandXDomain::Crypto,
        }
    }
}
