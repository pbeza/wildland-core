use crate::api;
use std::fmt::Display;
use wildland_corex::Identity;
pub use wildland_corex::{SeedPhraseWords, SEED_PHRASE_LEN};

#[derive(Clone, Debug)]
pub struct CryptoIdentity {
    identity_type: api::IdentityType,
    inner_identity: Identity,
}
impl Display for CryptoIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Type: {:?}
Private key: {}
Seed phrase: {}
",
            self.identity_type,
            self.inner_identity.get_xprv(),
            self.inner_identity.get_seed_phrase().join(" ")
        )
    }
}

impl CryptoIdentity {
    pub fn new(identity_type: api::IdentityType, inner_identity: Identity) -> Self {
        Self {
            identity_type,
            inner_identity,
        }
    }
}

impl api::Identity for CryptoIdentity {
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
}
