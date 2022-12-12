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

mod storage_backend;
mod template_params;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
pub use storage_backend::StorageBackendType;
pub use template_params::*;
use uuid::Uuid;

pub trait CloneSerializableTemplateBox {
    fn clone_box(&self) -> Box<dyn SerializableTemplate>;
}

impl Clone for Box<dyn SerializableTemplate> {
    fn clone(&self) -> Box<dyn SerializableTemplate> {
        self.clone_box()
    }
}

impl<T> CloneSerializableTemplateBox for T
where
    T: 'static + SerializableTemplate + Clone,
{
    fn clone_box(&self) -> Box<dyn SerializableTemplate> {
        Box::new(self.clone())
    }
}

#[typetag::serde]
pub trait SerializableTemplate: CloneSerializableTemplateBox + Debug {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StorageTemplate {
    name: Option<String>,
    uuid: Uuid,
    backend_type: StorageBackendType, // If we want to allow users to create their own custom templates and backends then this parameter should be a String
    #[serde(flatten)]
    template: Box<dyn SerializableTemplate>,
}

impl StorageTemplate {
    pub fn new(backend_type: StorageBackendType, template: Box<dyn SerializableTemplate>) -> Self {
        Self {
            name: None,
            uuid: Uuid::new_v4(),
            backend_type,
            template,
        }
    }

    pub fn with_name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn stringify(&self) -> String {
        format!("{:?}", &self)
    }

    pub fn backend_type(&self) -> StorageBackendType {
        self.backend_type
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
}
