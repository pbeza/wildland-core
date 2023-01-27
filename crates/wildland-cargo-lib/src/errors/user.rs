//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use thiserror::Error;
use wildland_corex::catlib_service::error::CatlibError;
use wildland_corex::{CryptoError, ForestIdentityCreationError, ForestRetrievalError, LssError};

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum CreateMnemonicError {
    #[error("Invalid Mnemonic words")]
    InvalidMnemonicWords,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum UserCreationError {
    #[error("Generic error: {0}")]
    Generic(String),
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Mnemonic generation error: {0}")]
    MnemonicGenerationError(CryptoError),
    #[error("Identity generation error: {0}")]
    IdentityGenerationError(CryptoError),
    #[error("Could not check if user already exists: {0}")]
    UserRetrievalError(UserRetrievalError),
    #[error("Could not create a new forest identity: {0}")]
    ForestIdentityCreationError(ForestIdentityCreationError),
    #[error(transparent)]
    LssError(#[from] LssError),
    #[error("Too low entropy")]
    EntropyTooLow,
    #[error("Generic error: {0}")]
    CatlibError(String),
}

impl From<CryptoError> for UserCreationError {
    fn from(crypto_err: CryptoError) -> Self {
        match &crypto_err {
            CryptoError::MnemonicGenerationError(_) => {
                UserCreationError::MnemonicGenerationError(crypto_err)
            }
            CryptoError::IdentityGenerationError(_) => {
                UserCreationError::IdentityGenerationError(crypto_err)
            }
            CryptoError::EntropyTooLow => UserCreationError::EntropyTooLow,
            _ => panic!(
                "Unexpected error happened while converting {crypto_err:?} into UserCreationError"
            ),
        }
    }
}

impl From<CatlibError> for UserCreationError {
    fn from(catlib_err: CatlibError) -> Self {
        match catlib_err {
            CatlibError::NoRecordsFound
            | CatlibError::MalformedDatabaseRecord
            | CatlibError::Generic(_) => UserCreationError::CatlibError(catlib_err.to_string()),
            CatlibError::RecordAlreadyExists => UserCreationError::UserAlreadyExists,
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum UserRetrievalError {
    #[error(transparent)]
    ForestRetrievalError(#[from] ForestRetrievalError),
    #[error("Default forest not found in LSS: {0}")]
    ForestNotFound(String),
    #[error(transparent)]
    LssError(#[from] LssError),
    #[error(transparent)]
    CatlibError(#[from] CatlibError),
    #[error("Metadata of this device has not been found in Forest")]
    DeviceMetadataNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error("Generic: {0}")]
    Generic(String),
}
