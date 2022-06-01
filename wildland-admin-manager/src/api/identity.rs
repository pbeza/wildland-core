use std::fmt::Display;

use wildland_corex::{CoreXError, SeedPhraseWords, WalletType};

#[derive(Clone, Copy, Debug)]
pub enum IdentityType {
    Master,
    Device,
}

pub trait Identity: Display + std::fmt::Debug + Send {
    // fn new(identity_type: IdentityType, name: String, inner_identity: &SeedPhrase) -> Self;
    fn get_identity_type(&self) -> IdentityType;
    fn get_pubkey(&self) -> Vec<u8>;
    fn get_fingerprint(&self) -> Vec<u8>;
    fn get_seed_phrase(&self) -> SeedPhraseWords;
    fn save(&self, wallet: WalletType) -> Result<(), CoreXError>;
}
