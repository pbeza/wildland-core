use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FoundationStorageTemplate {
    pub id: Uuid,
    pub credential_id: String,
    pub credential_secret: String,
    pub sc_url: String,
}
