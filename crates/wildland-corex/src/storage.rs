use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FoundationStorageTemplate {
    pub id: Uuid,
    pub credential_id: String,
    pub credential_secret: String,
    pub sc_url: String,
}

pub fn write_foundation_storage_template(fst: FoundationStorageTemplate) {}
