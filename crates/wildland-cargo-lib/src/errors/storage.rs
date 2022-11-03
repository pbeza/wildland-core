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
use wildland_corex::LssError;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum GetStoragesError {
    #[error("Get Storage error")]
    Error,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum DeleteStorageError {
    #[error("Delete Storage error")]
    Error,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum AddStorageError {
    #[error("Add Storage error")]
    Error,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GetStorageTemplateError {
    #[error(transparent)]
    LssError(#[from] LssError),
    #[error("Error while deserializing data retrieved from LSS: {0}")]
    DeserializationError(String),
}
