use super::{seed_phrase::SeedPhrase, AdminManagerResult, Identity};

pub trait AdminManager {
    /// Creates a master identity based on the provided seed phrase (whether it's a newly
    /// generated seed phrase or manually entered in the recovery flow.
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: &SeedPhrase,
    ) -> AdminManagerResult<&mut Box<dyn Identity>>;

    /// Creates a randomly generated seed phrase
    fn create_seed_phrase() -> AdminManagerResult<SeedPhrase>;

    fn get_master_identity(&mut self) -> &mut Option<Box<dyn Identity>>;
}
