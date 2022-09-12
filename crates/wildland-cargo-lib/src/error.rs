use thiserror::Error;
use wildland_corex::CoreXError;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CargoLibError {
    #[error("CoreX error")]
    CoreX(#[from] CoreXError),
}

impl CargoLibError {
    // TODO:WILX-205 error interface specification: what do we care about? do we want codes or some string kind?
    pub fn code(&self) -> u32 {
        match self {
            CargoLibError::CoreX(_inner) => 100, // TODO:WILX-205 codes
        }
    }
}
