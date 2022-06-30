use thiserror::Error;
use wildland_crypto::error::CryptoError;
use wildland_wallet::WalletError;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CoreXError {
    #[error("Saving identity error: {0}")]
    IdentitySaveError(WalletError),
    #[error("Could not create a wallet: {0}")]
    WalletCreationError(WalletError),
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
    #[error("CoreX error: {0}")]
    Generic(String),
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
