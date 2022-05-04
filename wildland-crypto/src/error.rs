//
// Wildland Project
//
// Copyright © 2021 Golem Foundation,
// 	    	     Piotr K. Isajew <piotr@wildland.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Generic error wrapper for Rust errors that need to propagate into
// the native bridge.

use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashMap;
use std::fmt;

#[derive(std::fmt::Debug)]
pub struct CargoError {
    error_type: &'static str,
    error_code: String,
    other_info: Option<HashMap<String, String>>,
}

pub trait CargoErrorRepresentable: Into<CargoError> {
    const CARGO_ERROR_TYPE: &'static str;
    fn error_code(&self) -> String;
    fn other_info(&self) -> Option<HashMap<String, String>> {
        None
    }
}

impl<T> From<T> for CargoError
where
    T: CargoErrorRepresentable,
{
    fn from(specific_error: T) -> CargoError {
        CargoError {
            error_type: T::CARGO_ERROR_TYPE,
            error_code: specific_error.error_code(),
            other_info: specific_error.other_info(),
        }
    }
}

impl Serialize for CargoError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CargoError", 3)?;
        state.serialize_field("errorType", &self.error_type)?;
        state.serialize_field("errorCode", &self.error_code)?;
        state.serialize_field("otherInfo", &self.other_info)?;
        state.end()
    }
}

impl fmt::Display for CargoError {
    /* Required as C++ binding only propagates string representation
    of error */
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
