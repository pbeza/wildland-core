use thiserror::Error;
use wildland_corex::CoreXError;

#[derive(Error, Debug, PartialEq)]
pub enum AdminManagerError {
    #[error("Email has been already verified")]
    EmailAlreadyVerified,
    #[error("Validation codes do not match")]
    ValidationCodesDoNotMatch,
    #[error("Email candidate not set")]
    EmailCandidateNotSet,
    #[error("CoreX error: {0}")]
    CoreX(CoreXError),
}

impl From<CoreXError> for AdminManagerError {
    fn from(corex_err: CoreXError) -> Self {
        AdminManagerError::CoreX(corex_err)
    }
}
