use serde::{Deserialize, Serialize};
use wildland_corex::storage::StorageTemplateTrait;

use super::foundation_storage::FoundationStorageTemplate;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "template")]
pub enum StorageTemplate {
    FoundationStorageTemplate(FoundationStorageTemplate),
}

impl StorageTemplate {
    pub fn stringify(&self) -> String {
        let uuid = match self {
            StorageTemplate::FoundationStorageTemplate(st) => st.uuid(),
        };
        format!(
            "Storage Template (uuid: {uuid}, type: {})",
            self.storage_template_type()
        )
    }

    fn storage_template_type(&self) -> &'static str {
        match self {
            StorageTemplate::FoundationStorageTemplate(_) => "Foundation Storage Template",
        }
    }
}

impl StorageTemplateTrait for StorageTemplate {
    fn uuid(&self) -> uuid::Uuid {
        match self {
            StorageTemplate::FoundationStorageTemplate(FoundationStorageTemplate {
                uuid, ..
            }) => *uuid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StorageTemplate;
    use crate::api::foundation_storage::FoundationStorageTemplate;
    use pretty_assertions::assert_eq;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn serialize_foundation_storage_template_as_json() {
        let fst = StorageTemplate::FoundationStorageTemplate(FoundationStorageTemplate {
            uuid: Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
            credential_id: "cred_id".to_owned(),
            credential_secret: "cred_secret".to_owned(),
            sc_url: "sc_url".to_owned(),
        });

        let expected = r#"
            {
                "type": "FoundationStorageTemplate",
                "template": {
                    "uuid": "00000000-0000-0000-0000-000000000001",
                    "credential_id": "cred_id",
                    "credential_secret": "cred_secret",
                    "sc_url": "sc_url"
                }
            }"#;

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(expected).unwrap(),
            serde_json::to_value(&fst).unwrap()
        );
    }

    #[test]
    fn serialize_foundation_storage_template_as_yaml() {
        let fst = StorageTemplate::FoundationStorageTemplate(FoundationStorageTemplate {
            uuid: Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
            credential_id: "cred_id".to_owned(),
            credential_secret: "cred_secret".to_owned(),
            sc_url: "sc_url".to_owned(),
        });

        let expected = r#"
            type: FoundationStorageTemplate
            template:
                uuid: 00000000-0000-0000-0000-000000000001
                credential_id: cred_id
                credential_secret: cred_secret
                sc_url: sc_url
        "#;

        assert_eq!(
            serde_yaml::from_str::<serde_yaml::Value>(expected).unwrap(),
            serde_yaml::to_value(&fst).unwrap()
        );
    }
}
