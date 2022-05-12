pub trait SeedPhrase {
    fn set_words(&mut self, words: Vec<String>);
    fn get_words(&self) -> Vec<String>;
}
