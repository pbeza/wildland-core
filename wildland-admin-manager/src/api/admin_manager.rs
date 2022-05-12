use super::{identity::Identity, seed_phrase::SeedPhrase};

pub trait AdminManager<S: SeedPhrase, I: Identity> {
    /// The method generates the seed phrase required to generate the master private key. Also
    /// based on the seed phrase the UI will present to the user the 12 words (the 12 words
    /// allow to restore the master key i.e. on other devices).
    fn generate_seed_phrase(&mut self) -> S;

    /// Creates the master identity based on the provided seed phrase (whether it's a newly
    /// generated seed phrase or manually entered in the recovery flow. The keys (ie. public
    /// private keypair) are stored in the Wallet component.
    fn create_master_identity_from_seed_phrase(seed: S) -> I;
}
