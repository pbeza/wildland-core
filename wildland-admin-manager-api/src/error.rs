use thiserror::Error;
use wildland_corex::CoreXError;

#[derive(Error, Debug)]
pub enum AdminManagerError {
    #[error("CoreX error: {0}")]
    CoreX(CoreXError),
}

impl From<CoreXError> for AdminManagerError {
    fn from(corex_err: CoreXError) -> Self {
        AdminManagerError::CoreX(corex_err)
    }
}
