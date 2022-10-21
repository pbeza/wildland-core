use thiserror::Error;
use wildland_corex::{
    CatlibError, CryptoError, ForestIdentityCreationError, ForestRetrievalError, LssError,
};

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum UserCreationError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Mnemonic generation error: {0}")]
    MnemonicGenerationError(CryptoError),
    #[error("Identity generation error: {0}")]
    IdentityGenerationError(CryptoError),
    #[error("Could not check if user already exists: {0}")]
    UserRetrievalError(UserRetrievalError),
    #[error("Could not create a new forest identity: {0}")]
    ForestIdentityCreationError(ForestIdentityCreationError),
    #[error(transparent)]
    LssError(#[from] LssError),
    #[error("Too low entropy")]
    EntropyTooLow,
    #[error("Generic error: {0}")]
    CatlibError(String),
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

impl From<CatlibError> for UserCreationError {
    fn from(catlib_err: CatlibError) -> Self {
        match catlib_err {
            CatlibError::NoRecordsFound
            | CatlibError::MalformedDatabaseEntry
            | CatlibError::Generic(_) => UserCreationError::CatlibError(catlib_err.to_string()),
            CatlibError::RecordAlreadyExists => UserCreationError::UserAlreadyExists,
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum UserRetrievalError {
    #[error(transparent)]
    ForestRetrievalError(#[from] ForestRetrievalError),
    #[error("Default forest not found in LSS: {0}")]
    ForestNotFound(String),
    #[error(transparent)]
    LssError(#[from] LssError),
    #[error(transparent)]
    CatlibError(#[from] CatlibError),
    #[error("Metadata of this device has not been found in Forest")]
    DeviceMetadataNotFound,
}
