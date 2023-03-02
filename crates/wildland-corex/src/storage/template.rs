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

use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use thiserror::Error;
use uuid::Uuid;

use super::StorageAccessMode;
use crate::catlib_service::error::CatlibError;
use crate::{ContainerPath, ErrContext, Storage};

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
    #[error("Template engine error: {0}")]
    TemplateEngineErr(String),
    #[error("Catlib error: {0}")]
    CatlibErr(String, CatlibError),
}

impl<T> ErrContext<T, StorageTemplateError> for Result<T, serde_json::Error> {
    fn context(self, ctx: impl Display) -> Result<T, StorageTemplateError> {
        self.map_err(|e| StorageTemplateError::SerdeErr(Self::format(e, ctx)))
    }
}
impl<T> ErrContext<T, StorageTemplateError> for Result<T, serde_yaml::Error> {
    fn context(self, ctx: impl Display) -> Result<T, StorageTemplateError> {
        self.map_err(|e| StorageTemplateError::SerdeErr(Self::format(e, ctx)))
    }
}
impl<T> ErrContext<T, StorageTemplateError> for Result<T, tera::Error> {
    fn context(self, ctx: impl Display) -> Result<T, StorageTemplateError> {
        self.map_err(|e| StorageTemplateError::TemplateEngineErr(Self::format(e, ctx)))
    }
}
impl<T> ErrContext<T, StorageTemplateError> for Result<T, CatlibError> {
    fn context(self, ctx: impl Display) -> Result<T, StorageTemplateError> {
        self.map_err(|e| StorageTemplateError::CatlibErr(ctx.to_string(), e))
    }
}

/// Storage Templates provide some general information about storage location. Their only purpose is to be
/// filled with the container's parameters during container creation and to generate Storage Manifest
/// (in opposition to a template it points to the storage location assigned to the particular container).
///
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "version")]
pub enum StorageTemplate {
    #[serde(rename = "1")]
    V1(StorageTemplateV1),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StorageTemplateV1 {
    /// If uuid is Some(_) then this template has been written to Catalog Backend which gave it an id.
    #[serde(skip)]
    uuid: Option<Uuid>,
    name: Option<String>,
    backend_type: String,
    template: serde_json::Value,
}

impl StorageTemplate {
    /// Creates new `StorageTemplate`
    ///
    /// # Args:
    /// - `backend_type` - string defining backend type
    /// - `template` - template specific fields ("template" property of a whole Storage Template object)
    ///
    pub fn try_new(
        backend_type: impl ToString,
        template: &impl Serialize,
    ) -> Result<Self, StorageTemplateError> {
        Ok(Self::V1(StorageTemplateV1 {
            name: None,
            uuid: None,
            backend_type: backend_type.to_string(),
            template: serde_json::to_value(template)
                .context("Error while deserializing 'template' property")?,
        }))
    }

    pub(crate) fn with_uuid(mut self, uuid: Uuid) -> Self {
        match &mut self {
            StorageTemplate::V1(template) => {
                template.uuid = Some(uuid);
                self
            }
        }
    }

    pub fn set_uuid(&mut self, uuid: Uuid) {
        match self {
            StorageTemplate::V1(template) => {
                template.uuid = Some(uuid);
            }
        }
    }

    pub fn name(&self) -> Option<String> {
        match &self {
            StorageTemplate::V1(template) => template.name.clone(),
        }
    }

    pub fn set_name(&mut self, name: String) {
        match self {
            StorageTemplate::V1(template) => template.name = Some(name),
        }
    }

    pub fn stringify(&self) -> String {
        format!("{:?}", &self)
    }

    /// Deserializes json-formatted content as a StorageTemplate.
    ///
    /// It expects the following structure:
    /// ```ignore
    /// {
    ///     "backend_type": "some type",
    ///     "template": {
    ///         ...template specific fields
    ///     }
    /// }
    /// ```
    pub fn from_json(content: Vec<u8>) -> Result<StorageTemplate, StorageTemplateError> {
        serde_json::from_slice(content.as_slice())
            .context("Error while deserializing StorageTemplate from json")
    }

    /// Deserializes yaml-formatted content as a StorageTemplate.
    ///
    /// It expects the following structure:
    /// ```ignore
    /// backend_type: some type
    /// template:
    ///     ...template specific fields
    /// ```
    pub fn from_yaml(content: Vec<u8>) -> Result<StorageTemplate, StorageTemplateError> {
        serde_yaml::from_slice(content.as_slice())
            .context("Error while deserializing StorageTemplate from yaml")
    }

    pub fn to_json(&self) -> Result<String, StorageTemplateError> {
        serde_json::to_string(&self).context("Error while converting template to json")
    }

    pub fn to_yaml(&self) -> Result<String, StorageTemplateError> {
        serde_yaml::to_string(&self).context("Error while converting template to yaml")
    }

    pub fn backend_type(&self) -> String {
        match &self {
            StorageTemplate::V1(template) => template.backend_type.clone(),
        }
    }

    /// Returns backend specific data under the `template` property
    pub fn template(&self) -> &serde_json::Value {
        match self {
            StorageTemplate::V1(tv1) => &tv1.template,
        }
    }

    /// If returned Some(_) then this template has been written to Catalog Backend which gave it an id.
    /// Otherwise it returns None
    pub fn uuid(&self) -> Option<Uuid> {
        match &self {
            StorageTemplate::V1(template) => template.uuid,
        }
    }

    /// If returned Some(_) then this template has been written to Catalog Backend which gave it an id.
    /// Otherwise it returns None
    pub fn uuid_str(&self) -> Option<String> {
        self.uuid().map(|u| u.to_string())
    }

    pub fn render(&self, params: TemplateContext) -> Result<Storage, StorageTemplateError> {
        let template_str = serde_json::to_string(&self.template())
            .context("Error deserializing template while rendering")?;
        let filled_template = Tera::one_off(
            &template_str,
            &Context::from_serialize(params)
                .context("Error while deserializing template params")?,
            true,
        )
        .context("Error while filling template with params")?;
        let storage_data: serde_json::Value = serde_json::from_str(&filled_template)
            .context("Deserialize Storage Error while rendering template")?;
        Ok(Storage::new(self.name(), self.backend_type(), storage_data))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::str::FromStr;

    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{StorageTemplate, StorageTemplateV1, TemplateContext};

    #[test]
    fn parse_generic_json_template() {
        let json_str = serde_json::json!({
            "version": "1",
            "template": {
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
                "with-index": false
            },
            "backend_type": "s3",
        })
        .to_string();

        let mut tpl = StorageTemplate::from_json(json_str.as_bytes().to_vec()).unwrap();

        assert_eq!(tpl.name(), None);

        tpl.set_name("random name".to_string());

        assert_eq!(tpl.name(), Some("random name".to_string()));
    }

    #[test]
    fn parse_generic_yaml_template() {
        let yaml_content = "
            version: '1'
            template:
                access:
                    - user1: '*'
                credentials:
                    access-key: NOT_SO_SECRET
                    secret-key: VERY_SECRET
                manifest-pattern:
                    path: /{path}.yaml
                    type: glob
                read-only: true
                s3_url: s3://michal-afc03a81-307c-4b41-b9dd-771835617900/{{ CONTAINER_UUID  }}
                with-index: false
            backend_type: s3
        ";

        let mut tpl = StorageTemplate::from_yaml(yaml_content.as_bytes().to_vec()).unwrap();

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

    #[test]
    fn test_to_json() {
        let template = StorageTemplate::V1(StorageTemplateV1 {
            uuid: Some(Uuid::from_u128(1)),
            name: Some("name".into()),
            backend_type: "backend type".into(),
            template: serde_json::to_value(HashMap::from([("a", "b")])).unwrap(),
        });

        let json = serde_json::to_value(&template).unwrap();
        let expected = serde_json::json!({
            "version": "1",
            "name": "name",
            "backend_type": "backend type",
            "template": {
                "a": "b"
            }
        });

        assert_eq!(json, expected);
    }

    #[test]
    fn test_to_yaml() {
        let template = StorageTemplate::V1(StorageTemplateV1 {
            uuid: Some(Uuid::from_u128(1)),
            name: Some("name".into()),
            backend_type: "backend type".into(),
            template: serde_json::to_value(HashMap::from([("a", "b")])).unwrap(),
        });

        let yaml = serde_yaml::to_value(&template).unwrap();
        let expected: serde_yaml::Value = serde_yaml::from_str(
            r#"
            version: '1'
            name: name
            backend_type: 'backend type'
            template:
                a: b
        "#,
        )
        .unwrap();

        assert_eq!(yaml, expected);
    }
}
