use super::{identity::Identity, AdminManagerResult, SeedPhraseWords};

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
}
