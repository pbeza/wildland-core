use std::fmt::Display;

use wildland_corex::{FingerPrint, SeedPhraseWords};

#[derive(Clone, Copy, Debug)]
pub enum IdentityType {
    Master,
    Device,
}

pub trait Identity: Display + std::fmt::Debug + Send {
    // fn new(identity_type: IdentityType, name: String, inner_identity: &SeedPhrase) -> Self;
    fn get_identity_type(&self) -> IdentityType;
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn get_pubkey(&self) -> Vec<u8>;
    fn get_fingerprint(&self) -> FingerPrint;
    fn get_seed_phrase(&self) -> SeedPhraseWords;
}
