use crate::api;
use std::fmt::Display;
use wildland_corex::Identity as CryptoIdentity;
pub use wildland_corex::{SeedPhraseWords, SEED_PHRASE_LEN};

#[derive(Clone, Debug)]
pub struct Identity {
    identity_type: api::IdentityType,
    name: String,
    inner_identity: CryptoIdentity,
}
impl Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Name: {}
Type: {:?}
Private key: {}
Seed phrase: {}
",
            self.name,
            self.identity_type,
            self.inner_identity.get_xprv(),
            self.inner_identity.get_seed_phrase().join(" ")
        )
    }
}

impl Identity {
    pub fn new(
        identity_type: api::IdentityType,
        name: String,
        inner_identity: CryptoIdentity,
    ) -> Self {
        Self {
            identity_type,
            name,
            inner_identity,
        }
    }
}

impl api::IdentityApi for Identity {
    fn get_pubkey(&self) -> Vec<u8> {
        todo!() // TODO
    }

    fn get_fingerprint(&self) -> Vec<u8> {
        todo!() // TODO
    }

    fn get_identity_type(&self) -> api::IdentityType {
        self.identity_type
    }

    fn get_seed_phrase(&self) -> SeedPhraseWords {
        self.inner_identity.get_seed_phrase()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
