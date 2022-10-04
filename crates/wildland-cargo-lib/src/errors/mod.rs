mod retrieval_error;
mod single_variant;
pub use retrieval_error::*;
pub use single_variant::*;

use wildland_corex::{CryptoError, ForestRetrievalError};
use wildland_http_client::error::WildlandHttpClientError;

use crate::{api::config::ParseConfigError, foundation_storage::FsaError, CargoLibCreationError};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum WildlandXDomain {
    ExternalServer,
    CargoUser,
    CargoConfig,
    Crypto,
    Lss,
    ExternalLibError,
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

impl ExceptionTrait for FsaError {
    fn reason(&self) -> String {
        self.to_string()
    }

    fn domain(&self) -> WildlandXDomain {
        match self {
            FsaError::StorageAlreadyExists => WildlandXDomain::CargoUser,
            FsaError::EvsError(inner) => match inner {
                WildlandHttpClientError::HttpError(_) => WildlandXDomain::ExternalServer,
                WildlandHttpClientError::CannotSerializeRequestError { .. } => {
                    WildlandXDomain::CargoUser
                }
                WildlandHttpClientError::CommonLibError(_) => WildlandXDomain::Crypto,
                WildlandHttpClientError::ReqwestError(_) => WildlandXDomain::ExternalLibError,
                WildlandHttpClientError::NoBody => WildlandXDomain::ExternalServer,
            },
            FsaError::CryptoError(_) => WildlandXDomain::Crypto,
            FsaError::InvalidCredentialsFormat(_) => WildlandXDomain::ExternalServer,
        }
    }
}
