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

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use wildland_corex::{
    StorageBackendType, StorageTemplate, StorageTemplateError, CONTAINER_NAME_PARAM, OWNER_PARAM,
};

use super::StorageCredentials;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundationStorageTemplate {
    bucket_uuid: Uuid,
    credential_id: String,
    credential_secret: String,
    sc_url: String,
    container_prefix: String,
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
            container_prefix: Self::default_container_prefix(),
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
            container_prefix: FoundationStorageTemplate::default_container_prefix(),
            credential_id,
            credential_secret,
            sc_url,
        }
    }

    fn default_container_prefix() -> String {
        format!("{{{{ {OWNER_PARAM} }}}}/{{{{ {CONTAINER_NAME_PARAM} }}}}")
    }
}

impl Display for FoundationStorageTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl TryFrom<FoundationStorageTemplate> for StorageTemplate {
    type Error = StorageTemplateError;
    fn try_from(fst: FoundationStorageTemplate) -> Result<Self, Self::Error> {
        StorageTemplate::try_new(StorageBackendType::FoundationStorage, fst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{collections::HashSet, str::FromStr};
    use uuid::Uuid;
    use wildland_corex::TemplateContext;

    #[test]
    fn serialize_foundation_storage_template_as_json() {
        let fst: StorageTemplate = FoundationStorageTemplate::new(
            Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "sc_url".to_owned(),
        )
        .try_into()
        .unwrap();
        let fst = fst.with_name("name");
        let uuid = fst.uuid();

        let expected = format!(
            r#"
            {{
                "uuid": "{uuid}",
                "backend_type": "FoundationStorage",
                "name": "name",
                "template": {{
                    "bucket_uuid": "00000000-0000-0000-0000-000000000001",
                    "credential_id": "cred_id",
                    "credential_secret": "cred_secret",
                    "sc_url": "sc_url",
                    "container_prefix": "{{{{ OWNER }}}}/{{{{ CONTAINER_NAME }}}}"
                }}
            }}"#
        );

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&expected).unwrap(),
            serde_json::to_value(&fst).unwrap()
        );
    }

    #[test]
    fn serialize_foundation_storage_template_as_yaml() {
        let fst: StorageTemplate = FoundationStorageTemplate::new(
            Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "sc_url".to_owned(),
        )
        .try_into()
        .unwrap();
        let fst = fst.with_name("name");
        let uuid = fst.uuid();

        let expected = format!(
            r#"
            name: name
            uuid: {uuid}
            backend_type: FoundationStorage
            template:
                bucket_uuid: 00000000-0000-0000-0000-000000000001
                credential_id: cred_id
                credential_secret: cred_secret
                sc_url: sc_url
                container_prefix: '{{{{ OWNER }}}}/{{{{ CONTAINER_NAME }}}}'
        "#
        );

        assert_eq!(
            serde_yaml::from_str::<serde_yaml::Value>(&expected).unwrap(),
            serde_yaml::to_value(&fst).unwrap()
        );
    }

    #[test]
    fn deserialize_foundation_storage_template_from_yaml() {
        let expected_template: StorageTemplate = FoundationStorageTemplate::new(
            Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "sc_url".to_owned(),
        )
        .try_into()
        .unwrap();
        let expected_template = expected_template.with_name("name");
        let uuid = expected_template.uuid();

        let yaml_template = format!(
            r#"
            name: name
            uuid: {uuid}
            backend_type: FoundationStorage
            template:
                bucket_uuid: 00000000-0000-0000-0000-000000000001
                credential_id: cred_id
                credential_secret: cred_secret
                sc_url: sc_url
                container_prefix: '{{{{ OWNER }}}}/{{{{ CONTAINER_NAME }}}}'
        "#
        );
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
                paths: HashSet::new(),
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
                container_prefix: Quentin Tarantino/Movies
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
