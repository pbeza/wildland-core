use thiserror::Error;
use wildland_corex::CoreXError;

pub type AdminManagerResult<T> = std::result::Result<T, AdminManagerError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AdminManagerError {
    #[error("Email has been already verified")]
    EmailAlreadyVerified,
    #[error("Validation codes do not match")]
    ValidationCodesDoNotMatch,
    #[error("Email candidate not set")]
    EmailCandidateNotSet,

    #[error("Error while parsing seed phrase: {0}")]
    ParseSeedPhraseError(String),

    #[error("CoreX error: {0}")]
    CoreX(CoreXError),
}

impl AdminManagerError {
    // TODO error interface specification: what do we care about? do we want codes or some string kind?
    pub fn code(&self) -> u32 {
        match self {
            AdminManagerError::CoreX(_inner) => 100, // TODO codes
            AdminManagerError::ParseSeedPhraseError(_) => 101,
            AdminManagerError::EmailAlreadyVerified => todo!(),
            AdminManagerError::ValidationCodesDoNotMatch => todo!(),
            AdminManagerError::EmailCandidateNotSet => todo!(), // TODO codes
        }
    }
}

impl From<CoreXError> for AdminManagerError {
    fn from(corex_err: CoreXError) -> Self {
        AdminManagerError::CoreX(corex_err)
    }
}
