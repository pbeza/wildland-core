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

use rustbreak::error::RustbreakError;
use thiserror::Error;

pub type CatlibResult<T> = Result<T, CatlibError>;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CatlibError {
    #[error("No records found")]
    NoRecordsFound,
    #[error("Corrupted database records")]
    MalformedDatabaseEntry,
    #[error("Entry already exists")]
    RecordAlreadyExists,
    #[error("Catlib error: {0}")]
    Generic(String),
}

impl From<RustbreakError> for CatlibError {
    fn from(rb_error: RustbreakError) -> Self {
        match rb_error {
            RustbreakError::DeSerialization(_) => {
                CatlibError::Generic("RustbreakError::DeSerialization".into())
            }
            RustbreakError::Poison => CatlibError::Generic("RustbreakError::Poison".into()),
            RustbreakError::Backend(_) => CatlibError::Generic("RustbreakError::Backend".into()),
            RustbreakError::WritePanic => CatlibError::Generic("RustbreakError::WritePanic".into()),
            _ => CatlibError::Generic("Unknown Rustbreak error".into()),
        }
    }
}
