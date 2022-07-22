use thiserror::Error;
use wildland_crypto::error::CryptoError;
use wildland_local_secure_storage::LSSError;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CoreXError {
    #[error("Seed phrase generation error: {0}")]
    SeedPhraseGenerationError(String),
    #[error("Identity generation error: {0}")]
    IdentityGenerationError(String),
    #[error("Identity read error: {0}")]
    IdentityReadError(String),
    #[error("Too low entropy")]
    EntropyTooLow,
    #[error("Seed phrase parsing error: {0}")]
    ParseSeedPhraseError(String),
    #[error("LSS Error: {0}")]
    LSSError(String),
    #[error("CoreX error: {0}")]
    Generic(String),
}

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
            CryptoError::SeedPhraseGenerationError(msg) => {
                CoreXError::SeedPhraseGenerationError(msg)
            }
            CryptoError::IdentityGenerationError(msg) => CoreXError::IdentityGenerationError(msg),
            CryptoError::EntropyTooLow => CoreXError::EntropyTooLow,
            CryptoError::KeyParsingError(_) => todo!(),
            CryptoError::MessageVerificationError(_) => todo!(),
            CryptoError::InvalidSignatureBytesError(_) => todo!(),
        }
    }
}
