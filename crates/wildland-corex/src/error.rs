use thiserror::Error;
use wildland_crypto::error::CryptoError;
use wildland_local_secure_storage::LSSError;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CoreXError {
    #[error("Cannot create forest identity: {0}")]
    CannotCreateForestIdentityError(String),
    #[error("Mnemonic generation error: {0}")]
    MnemonicGenerationError(String),
    #[error("Identity generation error: {0}")]
    IdentityGenerationError(String),
    #[error("Identity read error: {0}")]
    IdentityReadError(String),
    #[error("Too low entropy")]
    EntropyTooLow,
    #[error("LSS Error: {0}")]
    LSSError(String),
    #[error("CoreX error: {0}")]
    Generic(String),
}

/// Workaround for error types which don't implement `Clone` trait.
/// The error object needs to be cloned from the result object to be safely propagated through ffi bindings.
// TODO Remove it if https://wildlandio.atlassian.net/browse/WILX-135 is finished and exceptions are thrown in native platforms (Clone not needed anymore)
impl From<LSSError> for CoreXError {
    fn from(lss_err: LSSError) -> Self {
        match lss_err {
            LSSError::FileLSSError(err) => CoreXError::LSSError(format!("{:?}", err)),
        }
    }
}

impl From<CryptoError> for CoreXError {
    fn from(crypto_err: CryptoError) -> Self {
        match crypto_err {
            CryptoError::MnemonicGenerationError(msg) => CoreXError::MnemonicGenerationError(msg),
            CryptoError::IdentityGenerationError(msg) => CoreXError::IdentityGenerationError(msg),
            CryptoError::EntropyTooLow => CoreXError::EntropyTooLow,
            CryptoError::KeyParsingError(_) => todo!(),
            CryptoError::MessageVerificationError(_) => todo!(),
            CryptoError::InvalidSignatureBytesError(_) => todo!(),
        }
    }
}
