mod identity;
mod seed_phrase;

pub use identity::Identity;
pub use seed_phrase::SeedPhrase;

use crate::api;
use bip39::{Language, Mnemonic};

#[derive(Default)]
pub struct AdminManager<S: api::SeedPhrase> {
    seed_phrase: S,
}

impl api::AdminManager<SeedPhrase, Identity> for AdminManager<SeedPhrase> {
    fn generate_seed_phrase(&mut self) -> SeedPhrase {
        // TODO call crypto crate
        self.seed_phrase.0 = Mnemonic::generate_in(Language::English, 12)
            .unwrap()
            .word_iter()
            .map(|s| s.into())
            .collect();
        self.seed_phrase.clone()
    }

    fn create_master_identity_from_seed_phrase(seed: SeedPhrase) -> Identity {
        // TODO
        Identity::new_master_identity("hardcoded name".into(), vec![], vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::AdminManager;
    use crate::api::{AdminManager as AdminManagerApi, SeedPhrase};

    #[test]
    fn test_seed_phrase_len() {
        let mut admin_manager = AdminManager::default();
        let seed_phrase = admin_manager.generate_seed_phrase();
        assert_eq!(seed_phrase.get_words().len(), 12);
    }
}
