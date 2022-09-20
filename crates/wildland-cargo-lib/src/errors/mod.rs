mod creation_error;
mod retrieval_error;
mod user;

pub use creation_error::*;
pub use retrieval_error::*;
pub use user::*;

use wildland_corex::{CryptoError, ForestRetrievalError};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum WildlandXDomain {
    CargoUser,
    Crypto,
    _Catlib,
    _CoreX,
    _Dfs,
    Lss,
}

pub trait ExceptionTrait {
    fn reason(&self) -> String;
    fn domain(&self) -> WildlandXDomain;
}

pub trait ErrDomain {
    fn domain(&self) -> WildlandXDomain;
}

impl ErrDomain for CryptoError {
    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::Crypto
    }
}
impl ErrDomain for UserCreationError {
    fn domain(&self) -> WildlandXDomain {
        match self {
            UserCreationError::UserAlreadyExists => WildlandXDomain::CargoUser,
            UserCreationError::MnemonicGenerationError(_)
            | UserCreationError::IdentityGenerationError(_)
            | UserCreationError::ForestIdentityCreationError(_)
            | UserCreationError::EntropyTooLow
            | UserCreationError::ForestRetrievalError(ForestRetrievalError::KeypairParseError(_)) => {
                WildlandXDomain::Crypto
            }
            UserCreationError::LssError(_)
            | UserCreationError::ForestRetrievalError(ForestRetrievalError::LssError(_)) => {
                WildlandXDomain::Lss
            }
        }
    }
}
impl ErrDomain for wildland_corex::LssError {
    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::Lss
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
