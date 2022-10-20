use serde::{Deserialize, Serialize};
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StorageTemplateType {
    FoundationStorage,
}

trait StorageTemplateTrait {
    fn uuid(&self) -> Uuid;
    fn storage_template_type(&self) -> StorageTemplateType;
    fn data(&self) -> Vec<u8>;
}

#[derive(Clone)]
pub struct StorageTemplate {
    inner: Rc<dyn StorageTemplateTrait>,
}

impl StorageTemplate {
    pub fn uuid(&self) -> Uuid {
        self.inner.uuid()
    }

    pub fn storage_template_type(&self) -> StorageTemplateType {
        self.inner.storage_template_type()
    }

    pub fn data(&self) -> Vec<u8> {
        self.inner.data()
    }

    pub fn try_from_bytes(
        bytes: &[u8],
        storage_template_type: StorageTemplateType,
    ) -> Result<Self, String> {
        match storage_template_type {
            StorageTemplateType::FoundationStorage => Ok(Self {
                inner: Rc::new(
                    serde_json::from_slice::<FoundationStorageTemplate>(bytes)
                        .map_err(|e| e.to_string())?,
                ),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundationStorageTemplate {
    pub id: Uuid,
    pub credential_id: String,
    pub credential_secret: String,
    pub sc_url: String,
}

impl StorageTemplateTrait for FoundationStorageTemplate {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn storage_template_type(&self) -> StorageTemplateType {
        StorageTemplateType::FoundationStorage
    }

    fn data(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

impl From<FoundationStorageTemplate> for StorageTemplate {
    fn from(fst: FoundationStorageTemplate) -> Self {
        StorageTemplate {
            inner: Rc::new(fst),
        }
    }
}
