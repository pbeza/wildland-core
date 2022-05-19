use std::fmt::Display;
use wildland_admin_manager_api as api;
use wildland_crypto::identity as crypto_identity;

#[derive(Clone, Debug)]
pub struct Identity {
    identity_type: api::IdentityType,
    name: String,
    inner_identity: crypto_identity::Identity,
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
        inner_identity: crypto_identity::Identity,
    ) -> Self {
        Self {
            identity_type,
            name,
            inner_identity,
        }
    }
}

impl api::Identity for Identity {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_pubkey(&self) -> Vec<u8> {
        todo!() // TODO
    }

    fn get_fingerprint(&self) -> Vec<u8> {
        todo!() // TODO
    }

    fn get_identity_type(&self) -> api::IdentityType {
        self.identity_type
    }

    fn get_seed_phrase(&self) -> api::SeedPhraseWords {
        self.inner_identity.get_seed_phrase()
    }
}
