use super::wildland::{WildlandIdentity, WildlandIdentityApi, WildlandIdentityType};
use crate::{crypto::SeedPhrase, CoreXError};
use std::{fmt::Display, rc::Rc};
use wildland_crypto::identity::{Identity, SeedPhraseWords, SigningKeypair};
use wildland_wallet::Wallet;

pub trait MasterIdentityApi: Display {
    fn get_seed_phrase(&self) -> SeedPhraseWords;
    fn get_signing_keypair(&self) -> SigningKeypair;
    fn create_wildland_identity(
        &self,
        identity_type: WildlandIdentityType,
        name: String,
    ) -> Result<WildlandIdentity, CoreXError>;
}

type MasterIdentityWalletType = Rc<dyn Wallet>;
pub struct MasterIdentity {
    inner_identity: Identity,
    wallet: MasterIdentityWalletType,
}

impl MasterIdentity {
    pub fn new(wallet: MasterIdentityWalletType) -> Result<Self, CoreXError> {
        let seed = crate::generate_random_seed_phrase()
            .map_err(CoreXError::from)
            .map(SeedPhrase::from)?;

        let inner_identity =
            crate::try_identity_from_seed(seed.as_ref()).map_err(CoreXError::from)?;

        Ok(Self::with_identity(inner_identity, wallet))
    }

    pub fn with_identity(inner_identity: Identity, wallet: MasterIdentityWalletType) -> Self {
        Self {
            inner_identity,
            wallet,
        }
    }
}

impl MasterIdentityApi for MasterIdentity {
    fn get_seed_phrase(&self) -> SeedPhraseWords {
        self.inner_identity.get_seed_phrase()
    }

    fn get_signing_keypair(&self) -> SigningKeypair {
        self.inner_identity.signing_keypair()
    }

    fn create_wildland_identity(
        &self,
        identity_type: WildlandIdentityType,
        name: String,
    ) -> Result<WildlandIdentity, CoreXError> {
        let keypair = self.get_signing_keypair();
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
            self.inner_identity.get_seed_phrase().join(" ")
        )
    }
}
