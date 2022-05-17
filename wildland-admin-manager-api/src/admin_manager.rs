use super::{identity::Identity, AdminManagerResult, SeedPhraseWords};

pub trait AdminManager<I: Identity> {
    /// Creates a master identity based on the provided seed phrase (whether it's a newly
    /// generated seed phrase or manually entered in the recovery flow. The keys (ie. public
    /// private keypair) are stored in the Wallet component.
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: SeedPhraseWords,
    ) -> AdminManagerResult<I>;

    /// Creates a device identity based on the provided seed phrase (whether it's a newly
    /// generated seed phrase or manually entered in the recovery flow. The keys (ie. public
    /// private keypair) are stored in the Wallet component.
    fn create_device_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: SeedPhraseWords,
    ) -> AdminManagerResult<I>;

    /// Creates a randomly generated seed phrase
    fn create_seed_phrase() -> AdminManagerResult<SeedPhraseWords>;

    fn get_master_identity(&self) -> Option<I>;
}
