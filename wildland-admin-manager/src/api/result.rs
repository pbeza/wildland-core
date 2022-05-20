use thiserror::Error;
use wildland_corex::CoreXError;

pub type AdminManagerResult<T> = std::result::Result<T, AdminManagerError>;

#[derive(Error, Debug, Clone)]
pub enum AdminManagerError {
    #[error("CoreX error: {0}")]
    CoreX(CoreXError),
    #[error("Error while parsing seed phrase: {0}")]
    ParseSeedPhraseError(String),
}

impl AdminManagerError {
    pub fn code(&self) -> u32 {
        match self {
            AdminManagerError::CoreX(_inner) => 100, // TODO codes
            AdminManagerError::ParseSeedPhraseError(_) => 101, // TODO codes
        }
    }
}

impl From<CoreXError> for AdminManagerError {
    fn from(corex_err: CoreXError) -> Self {
        AdminManagerError::CoreX(corex_err)
    }
}
