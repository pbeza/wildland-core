use super::{AdminManagerResult, MasterIdentityApi, SeedPhrase, WildlandIdentityApi};
use std::sync::{Arc, Mutex};

pub type MasterIdentity = Arc<Mutex<dyn MasterIdentityApi>>;
pub type WildlandIdentity = Arc<Mutex<dyn WildlandIdentityApi>>;

pub struct IdentityPair {
    pub forest_id: WildlandIdentity,
    pub device_id: WildlandIdentity,
}

pub trait AdminManager {
    /// Creates a randomly generated seed phrase
    fn create_seed_phrase() -> AdminManagerResult<SeedPhrase>;

    /// Sends a 6-digit verification code to provided email address.
    fn send_verification_code(&mut self) -> AdminManagerResult<()>;

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
}
