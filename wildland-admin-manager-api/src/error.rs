use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdminManagerError {
    #[error("Seed phrase generation error: {0}")]
    SeedPhraseGenerationError(String),
    #[error("Identity generation error: {0}")]
    IdentityGenerationError(String),
    #[error("Too low entropy")]
    EntropyTooLow,
}
