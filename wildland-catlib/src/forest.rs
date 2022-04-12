use uuid::Uuid;

pub trait Forest {
    fn create_forest(&self, owner: &str, signers: &[&str], other_data: &[u8]) -> Result<Uuid, i32>;
    /// Implementation of adding a Signer will most likely update Containers access
    fn add_forest_signer(&self, forest_id: Uuid, signer: &str) -> Result<(), i32>;
    fn del_forest_signer(&self, forest_id: Uuid, signer: &str) -> Result<(), i32>;
    fn update_forest(&self, forest_id: Uuid, other_data: &[u8]) -> Result<(), i32>;
    fn remove_forest(&self, forest_id: Uuid) -> Result<(), i32>;
}
