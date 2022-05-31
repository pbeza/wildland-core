use super::{identity::Identity, AdminManagerResult};
use wildland_corex::SeedPhraseWords;

pub trait AdminManager {
    type Identity: Identity;

    /// Creates a master identity based on the provided seed phrase (whether it's a newly
    /// generated seed phrase or manually entered in the recovery flow.
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: SeedPhraseWords,
    ) -> AdminManagerResult<Self::Identity>;

    /// Creates a device identity based on the provided seed phrase (whether it's a newly
    /// generated seed phrase or manually entered in the recovery flow.
    fn create_device_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: SeedPhraseWords,
    ) -> AdminManagerResult<Self::Identity>;

    /// Creates a randomly generated seed phrase
    fn create_seed_phrase() -> AdminManagerResult<SeedPhraseWords>;

    fn get_master_identity(&self) -> Option<Self::Identity>;

    /// Sends a 6-digit verification code to provided email address.
    fn send_verification_code(&mut self) -> AdminManagerResult<()>;

    // Sets new unverified email
    fn set_email(&mut self, email: String);

    /// Checks whether verification code entered by a user is the same as generated one for a set email
    /// Returns error when email is not set
    fn verify_email(&mut self, verification_code: String) -> AdminManagerResult<()>;
}
