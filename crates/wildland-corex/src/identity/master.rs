use super::wildland::{WildlandIdentity, WildlandIdentityType};
use crate::CoreXError;
use std::{fmt::Display, rc::Rc};
use wildland_crypto::identity::{Identity, SigningKeypair};
use wildland_wallet::Wallet;

type MasterIdentityWalletType = Rc<dyn Wallet>;
pub struct MasterIdentity {
    inner_identity: Identity,
    wallet: MasterIdentityWalletType,
}

impl MasterIdentity {
    pub fn with_identity(inner_identity: Identity, wallet: MasterIdentityWalletType) -> Self {
        Self {
            inner_identity,
            wallet,
        }
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
        let identity = WildlandIdentity::new(identity_type, keypair, name, self.wallet.clone());

        identity.save()?;

        Ok(identity)
    }
}

impl Display for MasterIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
Type: Master
Seed phrase: {}
",
            self.inner_identity.get_mnemonic_phrase().join(" ")
        )
    }
}
