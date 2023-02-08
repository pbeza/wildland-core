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

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use thiserror::Error;
use uuid::Uuid;

use super::StorageAccessMode;
use crate::{ContainerPath, Storage};

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
    pub paths: Vec<ContainerPath>,
}

#[derive(Debug, Error, Clone)]
#[repr(C)]
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
    backend_type: String,
    template: serde_json::Value,
}

impl StorageTemplate {
    pub fn try_new(
        backend_type: impl ToString,
        template: &impl Serialize,
    ) -> Result<Self, StorageTemplateError> {
        Ok(Self {
            name: None,
            uuid: Uuid::new_v4(),
            backend_type: backend_type.to_string(),
            template: serde_json::to_value(template)
                .map_err(|e| StorageTemplateError::SerdeErr(e.to_string()))?,
        })
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn stringify(&self) -> String {
        format!("{:?}", &self)
    }

    pub fn from_json(content: Vec<u8>) -> Result<StorageTemplate, StorageTemplateError> {
        let storage_data: serde_json::Value = serde_json::from_slice(content.as_slice())
            .map_err(|e| StorageTemplateError::SerdeErr(e.to_string()))?;

        let backend_type = storage_data.get("backend_type").map_or(
            Err(StorageTemplateError::SerdeErr(
                "Missing backend_type key".into(),
            )),
            |b| {
                if b.is_string() {
                    Ok(b.as_str().unwrap())
                } else {
                    Err(StorageTemplateError::SerdeErr(
                        "Invalid backend_type value".into(),
                    ))
                }
            },
        )?;

        Self::try_new(backend_type, &storage_data)
    }

    pub fn from_yaml(content: Vec<u8>) -> Result<StorageTemplate, StorageTemplateError> {
        let storage_data: serde_yaml::Value = serde_yaml::from_slice(content.as_slice())
            .map_err(|e| StorageTemplateError::SerdeErr(e.to_string()))?;

        let backend_type = storage_data.get("backend_type").map_or(
            Err(StorageTemplateError::SerdeErr(
                "Missing backend_type key".into(),
            )),
            |b| {
                if b.is_string() {
                    Ok(b.as_str().unwrap())
                } else {
                    Err(StorageTemplateError::SerdeErr(
                        "Invalid backend_type value".into(),
                    ))
                }
            },
        )?;

        Self::try_new(backend_type, &storage_data)
    }

    pub fn to_json(&self) -> Result<String, StorageTemplateError> {
        serde_json::to_string(&self.template)
            .map_err(|e| StorageTemplateError::SerdeErr(e.to_string()))
    }

    pub fn to_yaml(&self) -> Result<String, StorageTemplateError> {
        serde_yaml::to_string(&self.template)
            .map_err(|e| StorageTemplateError::SerdeErr(e.to_string()))
    }

    pub fn backend_type(&self) -> String {
        self.backend_type.clone()
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn uuid_str(&self) -> String {
        self.uuid.to_string()
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
            self.backend_type.clone(),
            storage_data,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::str::FromStr;

    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{StorageTemplate, TemplateContext};

    #[test]
    fn parse_generic_json_template() {
        let json_str = serde_json::json!({
            "access":
            [
                {
                    "user": "*"
                }
            ],
            "credentials":
            {
                "access-key": "NOT_SO_SECRET",
                "secret-key": "VERY_SECRET"
            },
            "manifest-pattern":
            {
                "path": "/{path}.yaml",
                "type": "glob"
            },
            "read-only": true,
            "s3_url": "s3://michal-afc03a81-307c-4b41-b9dd-771835617900/{{ CONTAINER_UUID  }}",
            "backend_type": "s3",
            "with-index": false
        })
        .to_string();

        let mut tpl = StorageTemplate::from_json(json_str.as_bytes().to_vec()).unwrap();

        assert_eq!(tpl.name(), None);

        tpl.set_name("random name".to_string());

        assert_eq!(tpl.name(), Some("random name".to_string()));
    }

    #[test]
    fn test_rendering_template() {
        let storage_template = StorageTemplate::try_new(
            "FoundationStorage",
            &HashMap::from([
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
            paths: Vec::from_iter(["path1".into()]),
        };

        let rendered_storage = storage_template.render(params).unwrap();
        let uuid = rendered_storage.uuid();

        let expected_storage_toml = format!(
            r#"uuid = "{uuid}"
backend_type = "FoundationStorage"

[data]
field1 = "Some value with container name: Books"
"parameter in key: John Doe" = "enum: ReadOnly"
paths = "[path1]"
uuid = "00000000-0000-0000-0000-000000001111"
"#
        );

        assert_eq!(
            toml::Value::try_from(&rendered_storage).unwrap(),
            toml::Value::from_str(&expected_storage_toml).unwrap()
        );
    }
}
