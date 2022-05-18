use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AdminManagerError {
    #[error("Seed phrase generation error: {0}")]
    SeedPhraseGenerationError(String),
    #[error("Identity generation error: {0}")]
    IdentityGenerationError(String),
    #[error("Too low entropy")]
    EntropyTooLow,
    #[error("Email has been already verified")]
    EmailAlreadyVerified,
    #[error("Validation codes do not match")]
    ValidationCodesDoNotMatch,
    #[error("Email candidate not set")]
    EmailCandidateNotSet,
}
