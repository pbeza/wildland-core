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

mod template;

use derivative::Derivative;
use serde::{Deserialize, Serialize};
pub use template::*;
use uuid::Uuid;

/// Storage is basically the same struct as [`super::StorageTemplate`] but it serializable/deserializable content is filled with values provided by corex for a particular container
///
#[derive(Debug, Clone, Deserialize, Serialize, Eq, Derivative)]
#[derivative(Hash, PartialEq)]
pub struct Storage {
    name: Option<String>,
    uuid: Uuid,
    backend_type: String,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    data: serde_json::Value,
}

impl Storage {
    pub fn new(name: Option<String>, backend_type: String, data: serde_json::Value) -> Self {
        Self {
            name,
            uuid: Uuid::new_v4(),
            backend_type,
            data,
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn backend_type(&self) -> &str {
        self.backend_type.as_str()
    }

    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum StorageAccessMode {
    ReadWrite,
    ReadOnly,
}
