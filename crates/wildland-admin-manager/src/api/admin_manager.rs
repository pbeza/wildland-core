use wildland_corex::ManifestSigningKeypair;

use super::{AdminManagerResult, SeedPhrase};
use std::sync::{Arc, Mutex};
pub use wildland_corex::WildlandIdentity;

pub type WrappedWildlandIdentity = Arc<Mutex<WildlandIdentity>>;

#[derive(Clone, Debug)]
pub struct IdentityPair {
    pub forest_id: WrappedWildlandIdentity,
    pub device_id: WrappedWildlandIdentity,
}

impl IdentityPair {
    pub fn forest_id(&self) -> WrappedWildlandIdentity {
        self.forest_id.clone()
    }

    pub fn device_id(&self) -> WrappedWildlandIdentity {
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
