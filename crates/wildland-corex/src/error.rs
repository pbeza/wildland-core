use crate::LssError;
use thiserror::Error;
use wildland_crypto::error::CryptoError;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ForestRetrievalError {
    #[error(transparent)]
    LssError(#[from] LssError),
    #[error("Could not create keypair from bytes retrieved from LSS: {0}")]
    KeypairParseError(CryptoError),
}

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CoreXError {
    #[error("Cannot create forest identity: {0}")]
    CannotCreateForestIdentityError(String),
    #[error("Identity read error: {0}")]
    IdentityReadError(String),
    #[error("LSS Error: {0}")]
    LSSError(String),
    #[error("CoreX error: {0}")]
    Generic(String),
}

impl From<CryptoError> for CoreXError {
    #[tracing::instrument(level = "debug", ret)]
    fn from(crypto_err: CryptoError) -> Self {
        match crypto_err {
            CryptoError::KeyParsingError(_) => todo!(),
            CryptoError::MessageVerificationError(_) => todo!(),
            CryptoError::InvalidSignatureBytesError(_) => todo!(),
            _ => todo!(),
        }
    }
}
