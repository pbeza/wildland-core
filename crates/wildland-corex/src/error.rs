use crate::ForestIdentityCreationError;
use thiserror::Error;
use wildland_crypto::error::CryptoError;
use wildland_local_secure_storage::LssError;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum UserCreationError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Mnemonic generation error: {0}")]
    MnemonicGenerationError(CryptoError),
    #[error("Identity generation error: {0}")]
    IdentityGenerationError(CryptoError),
    #[error("Could not retrieve user's forest: {0}")]
    ForestRetrievalError(ForestRetrievalError),
    #[error("Could not create a new forest identity: {0}")]
    ForestIdentityCreationError(ForestIdentityCreationError),
    #[error(transparent)]
    LssError(#[from] LssError),
    #[error("Too low entropy")]
    EntropyTooLow,
}

impl From<CryptoError> for UserCreationError {
    #[tracing::instrument(level = "debug", ret)]
    fn from(crypto_err: CryptoError) -> Self {
        match &crypto_err {
            CryptoError::MnemonicGenerationError(_) => {
                UserCreationError::MnemonicGenerationError(crypto_err)
            }
            CryptoError::IdentityGenerationError(_) => {
                UserCreationError::IdentityGenerationError(crypto_err)
            }
            CryptoError::EntropyTooLow => UserCreationError::EntropyTooLow,
            _ => panic!(
                "Unexpected error happened while converting {crypto_err:?} into UserCreationError"
            ),
        }
    }
}

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
