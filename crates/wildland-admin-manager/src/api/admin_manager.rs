use wildland_corex::ManifestSigningKeypair;

use super::{AdminManagerResult, SeedPhrase, WildlandIdentityApi};
use std::sync::{Arc, Mutex};

pub type WildlandIdentity = Arc<Mutex<dyn WildlandIdentityApi>>;

#[derive(Clone, Debug)]
pub struct IdentityPair {
    pub forest_id: WildlandIdentity,
    pub device_id: WildlandIdentity,
}

impl IdentityPair {
    pub fn forest_id(&self) -> WildlandIdentity {
        self.forest_id.clone()
    }

    pub fn device_id(&self) -> WildlandIdentity {
        self.device_id.clone()
    }
}

pub trait AdminManagerApi {
    /// Creates a randomly generated seed phrase
    fn create_seed_phrase(&self) -> AdminManagerResult<SeedPhrase>;

    /// Sends a 6-digit verification code to provided email address.
    fn request_verification_email(&mut self) -> AdminManagerResult<()>;

    // Sets new unverified email
    fn set_email(&mut self, email: String);

    /// Checks whether verification code entered by a user is the same as generated one for a set email
    /// Returns error when email is not set
    fn verify_email(&mut self, verification_code: String) -> AdminManagerResult<()>;

    /// Create Forest and Device identities from given seedphrase. Ensures that the Device's
    /// name is stored in the Catalog.
    fn create_wildland_identities(
        &self,
        seed: &SeedPhrase,
        device_name: String,
    ) -> AdminManagerResult<IdentityPair>;

    // List all keypairs stored in wallet that Admin Manager was created with
    fn list_secrets(&self) -> AdminManagerResult<Vec<ManifestSigningKeypair>>;
}
