use thiserror::Error;
use wildland_crypto::error::CryptoError;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CoreXError {
    #[error("Seed phrase generation error: {0}")]
    SeedPhraseGenerationError(String),
    #[error("Identity generation error: {0}")]
    IdentityGenerationError(String),
    #[error("Too low entropy")]
    EntropyTooLow,
}

impl From<CryptoError> for CoreXError {
    fn from(crypto_err: CryptoError) -> Self {
        match crypto_err {
            CryptoError::SeedPhraseGenerationError(msg) => {
                CoreXError::SeedPhraseGenerationError(msg)
            }
            CryptoError::IdentityGenerationError(msg) => CoreXError::IdentityGenerationError(msg),
            CryptoError::EntropyTooLow => CoreXError::EntropyTooLow,
            CryptoError::CannotCreateKeyError(_) => todo!(),
            CryptoError::CannotEncryptMessageError(_) => todo!(),
            CryptoError::CannotDecryptMessageError(_) => todo!(),
            CryptoError::CannotVerifyMessageError(_) => todo!(),
            CryptoError::SignatureError(_) => todo!(),
        }
    }
}
