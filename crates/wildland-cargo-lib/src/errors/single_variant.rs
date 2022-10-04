use super::{ErrDomain, ExceptionTrait, WildlandXDomain};
use std::fmt::Display;

pub type SingleErrVariantResult<T, E> = Result<T, SingleVariantError<E>>;
#[derive(Debug, Clone)]
#[repr(C)]
pub enum SingleVariantError<T: Clone> {
    Failure(T),
}

impl<E: Display + Clone + ErrDomain> ExceptionTrait for SingleVariantError<E> {
    fn reason(&self) -> String {
        match self {
            Self::Failure(e) => e.to_string(),
        }
    }

    fn domain(&self) -> WildlandXDomain {
        match self {
            SingleVariantError::Failure(e) => e.domain(),
        }
    }
}
