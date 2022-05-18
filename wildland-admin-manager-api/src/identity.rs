use super::SeedPhraseWords;

#[derive(Clone, Copy, Debug)]
pub enum IdentityType {
    Master,
    Device,
}

pub trait Identity: Clone {
    fn get_identity_type(&self) -> IdentityType;
    fn get_name(&self) -> String;
    fn get_pubkey(&self) -> Vec<u8>;
    fn get_fingerprint(&self) -> Vec<u8>;
    fn get_seed_phrase(&self) -> SeedPhraseWords;
}
