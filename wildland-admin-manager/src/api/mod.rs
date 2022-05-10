pub trait SeedPhrase {
    fn set_words(&mut self, words: Vec<String>);
    fn get_words(&self) -> Vec<String>;
}

pub trait AdminManager<S: SeedPhrase> {
    /// The method generates the seed phrase required to generate the master private key. Also
    /// based on the seed phrase the UI will present to the user the 12 words (the 12 words
    /// allow to restore the master key i.e. on other devices).
    fn generate_seed_phrase(&mut self) -> S;
}
