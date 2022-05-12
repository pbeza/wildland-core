use crate::api;

#[derive(Default, Clone)]
pub struct SeedPhrase(pub Vec<String>);

impl api::SeedPhrase for SeedPhrase {
    fn set_words(&mut self, words: Vec<String>) {
        self.0 = words
    }

    fn get_words(&self) -> Vec<String> {
        self.0.clone()
    }
}
