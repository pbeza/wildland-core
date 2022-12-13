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

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tera::{Context, Tera};
use uuid::Uuid;

use super::StorageAccessMode;
use crate::{Storage, StorageBackendType};

pub const CONTAINER_NAME_PARAM: &str = "CONTAINER_NAME";
pub const OWNER_PARAM: &str = "OWNER";
pub const ACCESS_MODE_PARAM: &str = "ACCESS_MODE"; // read-write / readonly

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    #[serde(rename = "CONTAINER_NAME")]
    pub container_name: String,
    #[serde(rename = "OWNER")]
    pub owner: String,
    #[serde(rename = "ACCESS_MODE")]
    pub access_mode: StorageAccessMode,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StorageTemplate {
    name: Option<String>,
    uuid: Uuid,
    backend_type: StorageBackendType, // If we want to allow users to create their own custom templates and backends then this parameter should be a String
    template: serde_json::Value,
}

impl StorageTemplate {
    pub fn new(backend_type: StorageBackendType, template: impl Serialize) -> Self {
        Self {
            name: None,
            uuid: Uuid::new_v4(),
            backend_type,
            template: serde_json::to_value(&template).unwrap(), // TODO unwrap
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

    pub fn render(&self, params: TemplateContext) -> Storage {
        let template_str = serde_json::to_string(&self.template).unwrap(); // TODO unwrap
        let filled_template = Tera::one_off(
            &template_str,                             // TODO unwrap
            &Context::from_serialize(params).unwrap(), // TODO unwrap
            true,
        )
        .unwrap(); // TODO unwrap
        let storage_data: serde_json::Value = serde_json::from_str(&filled_template).unwrap(); // TODO unwrap
        Storage::new(self.name.clone(), self.backend_type, storage_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::{StorageBackendType, StorageTemplate, TemplateContext};
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn test_rendering_template() {
        let storage_template = StorageTemplate::new(
            StorageBackendType::FoundationStorage,
            HashMap::from([
                (
                    "field1".to_owned(),
                    "Some value with container name: {{ CONTAINER_NAME }}".to_string(),
                ),
                (
                    "parameter in key: {{ OWNER }}".to_owned(),
                    "enum: {{ ACCESS_MODE }}".to_string(),
                ),
            ]),
        );

        let params = TemplateContext {
            container_name: "Books".to_string(),
            owner: "John Doe".to_string(),
            access_mode: crate::StorageAccessMode::ReadOnly,
        };

        let rendered_storage = storage_template.render(params);
        let uuid = rendered_storage.uuid();

        let expected_storage_toml = format!(
            r#"uuid = "{uuid}"
backend_type = "FoundationStorage"

[data]
field1 = "Some value with container name: Books"
"parameter in key: John Doe" = "enum: ReadOnly"
"#
        );

        assert_eq!(
            toml::to_string(&rendered_storage).unwrap(),
            expected_storage_toml
        );
    }
}
