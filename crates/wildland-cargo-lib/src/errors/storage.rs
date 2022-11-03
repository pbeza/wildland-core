use thiserror::Error;
use wildland_corex::LssError;

use super::ExceptionTrait;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GetStoragesError {}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DeleteStorageError {}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AddStorageError {}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GetStorageTemplateError {
    #[error(transparent)]
    LssError(#[from] LssError),
    #[error("Error while deserializing data retrieved from LSS: {0}")]
    DeserializationError(String),
}

impl ExceptionTrait for GetStorageTemplateError {
    fn reason(&self) -> String {
        self.to_string()
    }
}
