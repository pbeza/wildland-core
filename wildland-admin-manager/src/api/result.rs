use thiserror::Error;
use wildland_corex::CoreXError;

pub type AdminManagerResult<T> = std::result::Result<T, AdminManagerError>;

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
