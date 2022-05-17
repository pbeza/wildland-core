use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdminManagerError {
    #[error("Wallet error: {0}")]
    Wallet(WalletError),
    #[error("CoreX error: {0}")]
    CoreX(CoreXError),
    #[error("Email candidate not set")]
    EmailCandidateNotSet,
    #[error("Email has been already verified")]
    EmailAlreadyVerified,
    #[error("Validation codes do not match")]
    ValidationCodesDoNotMatch,
}

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("TODO wallet errors")]
    Error1, // TODO
}

#[derive(Debug, Error)]
pub enum CoreXError {
    #[error("Catlib error: {0}")]
    Catlib(CatlibError),
    #[error("DFS error: {0}")]
    Dfs(DfsError),
    #[error("Crypto error: {0}")]
    Crypto(CryptoError),
}

#[derive(Debug, Error)]
pub enum CatlibError {
    #[error("TODO catlib errors")]
    SomeCatlibError1, // TODO
}

#[derive(Debug, Error)]
pub enum DfsError {
    #[error("TODO dfs errors")]
    SomeDfsError, // TODO
}

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Seed phrase generation error: {0}")]
    SeedPhraseGenerationError(String),
    #[error("Identity generation error: {0}")]
    IdentityGenerationError(String),
    #[error("Too low entropy")]
    EntropyTooLow,
}
