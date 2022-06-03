use super::{seed_phrase::SeedPhrase, AdminManagerResult, Identity};
use std::sync::{Arc, Mutex};

pub type AdminManagerIdentity = Arc<Mutex<dyn Identity>>;

pub trait AdminManager {
    /// Creates a master identity based on the provided seed phrase (whether it's a newly
    /// generated seed phrase or manually entered in the recovery flow.
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: &SeedPhrase,
    ) -> AdminManagerResult<AdminManagerIdentity>;

    fn get_master_identity(&self) -> Option<AdminManagerIdentity>;

    /// Sends a 6-digit verification code to provided email address.
    fn send_verification_code(&mut self) -> AdminManagerResult<()>;

    // Sets new unverified email
    fn set_email(&mut self, email: String);

    /// Checks whether verification code entered by a user is the same as generated one for a set email
    /// Returns error when email is not set
    fn verify_email(&mut self, verification_code: String) -> AdminManagerResult<()>;
}
