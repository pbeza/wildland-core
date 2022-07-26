use super::wildland::{WildlandIdentity, WildlandIdentityType};
use crate::CoreXError;
use std::fmt::Display;
use wildland_crypto::identity::{Identity, SigningKeypair};

pub struct MasterIdentity {
    inner_identity: Identity,
}

impl MasterIdentity {
    pub fn with_identity(inner_identity: Identity) -> Self {
        Self { inner_identity }
    }

    pub fn get_forest_keypair(&self) -> SigningKeypair {
        self.inner_identity.forest_keypair(0)
    }

    pub fn create_wildland_identity(
        &self,
        identity_type: WildlandIdentityType,
        name: String,
    ) -> Result<WildlandIdentity, CoreXError> {
        let keypair = self.get_forest_keypair();
        let identity = WildlandIdentity::new(identity_type, keypair, name);

        Ok(identity)
    }
}

impl Display for MasterIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Type: Master
Mnemonic: {}
",
            self.inner_identity.get_mnemonic().join(" ")
        )
    }
}
