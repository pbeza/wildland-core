use crate::api;
use bip39::{Language, Mnemonic};

#[derive(Default, Clone)]
pub struct SeedPhrase(Vec<String>);

impl api::SeedPhrase for SeedPhrase {
    fn set_words(&mut self, words: Vec<String>) {
        self.0 = words
    }

    fn get_words(&self) -> Vec<String> {
        self.0.clone()
    }
}

#[derive(Default)]
pub struct AdminManager<S: api::SeedPhrase> {
    seed_phrase: S,
}

impl api::AdminManager<SeedPhrase> for AdminManager<SeedPhrase> {
    fn generate_seed_phrase(&mut self) -> SeedPhrase {
        self.seed_phrase.0 = Mnemonic::generate_in(Language::English, 12)
            .unwrap()
            .word_iter()
            .map(|s| s.into())
            .collect();
        self.seed_phrase.clone()
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
