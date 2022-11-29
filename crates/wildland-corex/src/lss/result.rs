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
use wasm_bindgen::JsValue;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
#[error("Local Secure Storage error: {0}")]
pub enum LssError {
    Error(String),
}

impl From<JsValue> for LssError {
    fn from(json: JsValue) -> Self {
        LssError::Error(
            json.as_string()
                .unwrap_or("JS didn't provide an error message!".to_string()),
        )
    }
}

pub type LssResult<T> = Result<T, LssError>;
