use thiserror::Error;
use wildland_corex::CatlibError;

use super::ExceptionTrait;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ContainerMountError {}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ContainerUnmountError {}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ContainerDeletionError {
    #[error(transparent)]
    CatlibError(#[from] CatlibError),
    #[error("Could not lock mutex on Container's data: {0}")]
    ContainerDataLockError(String),
}

impl ExceptionTrait for ContainerDeletionError {
    fn reason(&self) -> String {
        self.to_string()
    }
}
