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
use thiserror::Error;
use uuid::Uuid;

use super::StorageAccessMode;
use crate::{Storage, StorageBackendType};

pub const CONTAINER_NAME_PARAM: &str = "CONTAINER_NAME";
pub const OWNER_PARAM: &str = "OWNER";
pub const ACCESS_MODE_PARAM: &str = "ACCESS_MODE"; // read-write / readonly
pub const CONTAINER_UUID_PARAM: &str = "CONTAINER_UUID";
pub const PATHS_PARAM: &str = "PATHS";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    #[serde(rename = "CONTAINER_NAME")]
    pub container_name: String,
    #[serde(rename = "OWNER")]
    pub owner: String,
    #[serde(rename = "ACCESS_MODE")]
    pub access_mode: StorageAccessMode,
    #[serde(rename = "CONTAINER_UUID")]
    pub container_uuid: Uuid,
    #[serde(rename = "PATHS")]
    pub paths: Vec<String>,
}

#[derive(Debug, Error, Clone)]
pub enum StorageTemplateError {
    #[error("Ser/deserialization error: {0}")]
    SerdeErr(String),
    #[error("Template engine error : {0}")]
    TemplateEngineErr(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StorageTemplate {
    name: Option<String>,
    uuid: Uuid,
    backend_type: StorageBackendType, // If we want to allow users to create their own custom templates and backends then this parameter should be a String
    template: serde_json::Value,
}

impl StorageTemplate {
    pub fn try_new(
        backend_type: StorageBackendType,
        template: impl Serialize,
    ) -> Result<Self, StorageTemplateError> {
        Ok(Self {
            name: None,
            uuid: Uuid::new_v4(),
            backend_type,
            template: serde_json::to_value(&template)
                .map_err(|e| StorageTemplateError::SerdeErr(e.to_string()))?,
        })
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

    pub fn render(&self, params: TemplateContext) -> Result<Storage, StorageTemplateError> {
        let template_str = serde_json::to_string(&self.template)
            .map_err(|e| StorageTemplateError::SerdeErr(e.to_string()))?;
        let filled_template = Tera::one_off(
            &template_str,
            &Context::from_serialize(params)
                .map_err(|e| StorageTemplateError::TemplateEngineErr(e.to_string()))?,
            true,
        )
        .map_err(|e| StorageTemplateError::TemplateEngineErr(e.to_string()))?;
        let storage_data: serde_json::Value = serde_json::from_str(&filled_template)
            .map_err(|e| StorageTemplateError::SerdeErr(e.to_string()))?;
        Ok(Storage::new(
            self.name.clone(),
            self.backend_type,
            storage_data,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{StorageBackendType, StorageTemplate, TemplateContext};
    use pretty_assertions::assert_eq;
    use std::{collections::HashMap, str::FromStr};
    use uuid::Uuid;

    #[test]
    fn test_rendering_template() {
        let storage_template = StorageTemplate::try_new(
            StorageBackendType::FoundationStorage,
            HashMap::from([
                (
                    "field1".to_owned(),
                    "Some value with container name: {{ CONTAINER_NAME }}".to_owned(),
                ),
                (
                    "parameter in key: {{ OWNER }}".to_owned(),
                    "enum: {{ ACCESS_MODE }}".to_owned(),
                ),
                ("uuid".to_owned(), "{{ CONTAINER_UUID }}".to_owned()),
                ("paths".to_owned(), "{{ PATHS }}".to_owned()),
            ]),
        )
        .unwrap();

        let params = TemplateContext {
            container_name: "Books".to_owned(),
            owner: "John Doe".to_owned(),
            access_mode: crate::StorageAccessMode::ReadOnly,
            container_uuid: Uuid::from_str("00000000-0000-0000-0000-000000001111").unwrap(),
            paths: vec!["path1".to_owned(), "path2".to_owned()],
        };

        let rendered_storage = storage_template.render(params).unwrap();
        let uuid = rendered_storage.uuid();

        let expected_storage_toml = format!(
            r#"uuid = "{uuid}"
backend_type = "FoundationStorage"

[data]
field1 = "Some value with container name: Books"
"parameter in key: John Doe" = "enum: ReadOnly"
paths = "[path1, path2]"
uuid = "00000000-0000-0000-0000-000000001111"
"#
        );

        assert_eq!(
            toml::Value::try_from(&rendered_storage).unwrap(),
            toml::Value::from_str(&expected_storage_toml).unwrap()
        );
    }
}
