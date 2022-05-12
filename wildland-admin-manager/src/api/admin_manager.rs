use super::{identity::Identity, SeedPhraseWords};

pub trait AdminManager<I: Identity> {
    /// Creates a master identity based on the provided seed phrase (whether it's a newly
    /// generated seed phrase or manually entered in the recovery flow. The keys (ie. public
    /// private keypair) are stored in the Wallet component.
    fn create_master_identity_from_seed_phrase(&mut self, name: String, seed: SeedPhraseWords)
        -> I;

    /// Creates a master identity based on the generated seed phrase
    fn create_master_identity(&mut self, name: String) -> I;

    /// Creates a device identity based on the provided seed phrase (whether it's a newly
    /// generated seed phrase or manually entered in the recovery flow. The keys (ie. public
    /// private keypair) are stored in the Wallet component.
    fn create_device_identity_from_seed_phrase(&mut self, name: String) -> I;

    /// Creates a device identity based on the generated seed phrase
    fn create_device_identity(&mut self, name: String, seed: SeedPhraseWords) -> I;

    fn get_identity(&self) -> I;
}
