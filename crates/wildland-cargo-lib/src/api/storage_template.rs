use serde::{Deserialize, Serialize};
use wildland_corex::storage::StorageTemplateTrait;

use super::foundation_storage::FoundationStorageTemplate;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
                id, ..
            }) => *id,
        }
    }
}
