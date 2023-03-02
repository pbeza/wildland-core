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
use uuid::Uuid;
use wildland_corex::{StorageTemplate, StorageTemplateError, CONTAINER_NAME_PARAM, OWNER_PARAM};

use crate::api::foundation_storage::StorageCredentials;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundationStorageTemplate {
    bucket_uuid: Uuid,
    credential_id: String,
    credential_secret: String,
    sc_url: String,
    container_dir: String,
}

impl FoundationStorageTemplate {
    #[cfg(test)]
    pub fn new(
        uuid: Uuid,
        credential_id: String,
        credential_secret: String,
        sc_url: String,
    ) -> Self {
        Self {
            bucket_uuid: uuid,
            credential_id,
            credential_secret,
            sc_url,
            container_dir: Self::default_container_dir(),
        }
    }

    pub fn from_storage_credentials_and_sc_url(
        StorageCredentials {
            id,
            credential_id,
            credential_secret,
        }: StorageCredentials,
        sc_url: String,
    ) -> FoundationStorageTemplate {
        FoundationStorageTemplate {
            bucket_uuid: id,
            container_dir: FoundationStorageTemplate::default_container_dir(),
            credential_id,
            credential_secret,
            sc_url,
        }
    }

    fn default_container_dir() -> String {
        format!("{{{{ {OWNER_PARAM} }}}}/{{{{ {CONTAINER_NAME_PARAM} }}}}")
    }
}

impl TryFrom<FoundationStorageTemplate> for StorageTemplate {
    type Error = StorageTemplateError;
    fn try_from(fst: FoundationStorageTemplate) -> Result<Self, Self::Error> {
        StorageTemplate::try_new("FoundationStorage", &fst)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;
    use uuid::Uuid;
    use wildland_corex::TemplateContext;

    use super::*;

    #[test]
    fn serialize_foundation_storage_template_as_json() {
        let mut fst: StorageTemplate = FoundationStorageTemplate::new(
            Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "sc_url".to_owned(),
        )
        .try_into()
        .unwrap();
        fst.set_name("name".into());

        let expected = serde_json::json!(
            {
                "version": "1",
                "backend_type": "FoundationStorage",
                "name": "name",
                "template": {
                    "bucket_uuid": "00000000-0000-0000-0000-000000000001",
                    "credential_id": "cred_id",
                    "credential_secret": "cred_secret",
                    "sc_url": "sc_url",
                    "container_dir": "{{ OWNER }}/{{ CONTAINER_NAME }}"
                }
            }
        );

        assert_eq!(expected, serde_json::to_value(&fst).unwrap());
    }

    #[test]
    fn serialize_foundation_storage_template_as_yaml() {
        let mut fst: StorageTemplate = FoundationStorageTemplate::new(
            Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "sc_url".to_owned(),
        )
        .try_into()
        .unwrap();
        fst.set_name("name".into());

        let expected = r#"
            version: '1'
            name: name
            backend_type: FoundationStorage
            template:
                bucket_uuid: 00000000-0000-0000-0000-000000000001
                credential_id: cred_id
                credential_secret: cred_secret
                sc_url: sc_url
                container_dir: '{{ OWNER }}/{{ CONTAINER_NAME }}'
        "#;

        assert_eq!(
            serde_yaml::from_str::<serde_yaml::Value>(&expected).unwrap(),
            serde_yaml::to_value(&fst).unwrap()
        );
    }

    #[test]
    fn deserialize_foundation_storage_template_from_yaml() {
        let mut expected_template: StorageTemplate = FoundationStorageTemplate::new(
            Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "sc_url".to_owned(),
        )
        .try_into()
        .unwrap();
        expected_template.set_name("name".into());

        let yaml_template = r#"
            version: 1
            name: name
            backend_type: FoundationStorage
            template:
                bucket_uuid: 00000000-0000-0000-0000-000000000001
                credential_id: cred_id
                credential_secret: cred_secret
                sc_url: sc_url
                container_dir: '{{ OWNER }}/{{ CONTAINER_NAME }}'
        "#;
        assert_eq!(
            serde_yaml::to_value(serde_yaml::from_str::<StorageTemplate>(&yaml_template).unwrap())
                .unwrap(),
            serde_yaml::to_value(&expected_template).unwrap()
        );
    }

    #[test]
    fn render_foundation_storage_template() {
        let fst: StorageTemplate = FoundationStorageTemplate::new(
            Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "sc_url".to_owned(),
        )
        .try_into()
        .unwrap();

        let storage = fst
            .render(TemplateContext {
                container_name: "Movies".to_owned(),
                owner: "Quentin Tarantino".to_owned(),
                access_mode: wildland_corex::StorageAccessMode::ReadWrite,
                container_uuid: Uuid::new_v4(),
                paths: Vec::new(),
            })
            .unwrap();
        let storage_uuid = storage.uuid();

        let expected_storage_yaml: serde_yaml::Value = serde_yaml::from_str(&format!(
            r#"
            name: null
            uuid: {storage_uuid}
            backend_type: FoundationStorage
            data:
                bucket_uuid: 00000000-0000-0000-0000-000000000001
                container_dir: Quentin Tarantino/Movies
                credential_id: cred_id
                credential_secret: cred_secret
                sc_url: sc_url
        "#
        ))
        .unwrap();

        assert_eq!(
            expected_storage_yaml,
            serde_yaml::to_value(&storage).unwrap()
        );
    }
}
