mod creation_error;
mod retrieval_error;

pub use creation_error::*;
pub use retrieval_error::*;

use wildland_corex::{CryptoError, ForestRetrievalError};

use crate::{api::config::ParseConfigError, CargoLibCreationError};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum WildlandXDomain {
    CargoUser,
    CargoConfig,
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

impl ErrDomain for CryptoError {
    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::Crypto
    }
}
impl ErrDomain for wildland_corex::UserCreationError {
    fn domain(&self) -> WildlandXDomain {
        match self {
            wildland_corex::UserCreationError::UserAlreadyExists => WildlandXDomain::CargoUser,
            wildland_corex::UserCreationError::MnemonicGenerationError(_)
            | wildland_corex::UserCreationError::IdentityGenerationError(_)
            | wildland_corex::UserCreationError::ForestIdentityCreationError(_)
            | wildland_corex::UserCreationError::EntropyTooLow
            | wildland_corex::UserCreationError::ForestRetrievalError(
                ForestRetrievalError::KeypairParseError(_),
            ) => WildlandXDomain::Crypto,
            wildland_corex::UserCreationError::LssError(_)
            | wildland_corex::UserCreationError::ForestRetrievalError(
                ForestRetrievalError::LssError(_),
            ) => WildlandXDomain::Lss,
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

impl ErrDomain for CargoLibCreationError {
    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::CargoConfig
    }
}

impl ErrDomain for ParseConfigError {
    fn domain(&self) -> WildlandXDomain {
        WildlandXDomain::CargoConfig
    }
}
