use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use super::wildland::{WildlandIdentity, WildlandIdentityApi, WildlandIdentityType};
use wildland_crypto::identity::{Identity, SeedPhraseWords};
use wildland_wallet::WalletFactory;

use crate::crypto::SeedPhrase;
use crate::{CoreXError, CryptoSigningKeypair};

pub trait MasterIdentityApi: Display {
    fn get_seed_phrase(&self) -> SeedPhraseWords;
    fn get_signing_keypair(&self) -> Box<dyn CryptoSigningKeypair>;
    fn create_wildland_identity(
        &self,
        identity_type: WildlandIdentityType,
        name: String,
    ) -> Result<Arc<Mutex<dyn WildlandIdentityApi>>, CoreXError>;
}

#[derive(Clone)]
pub struct MasterIdentity<W: WalletFactory> {
    inner_identity: Identity,
    wallet: W,
}

impl<W: WalletFactory + 'static> MasterIdentity<W> {
    pub fn default() -> Result<Self, CoreXError> {
        let seed = crate::generate_random_seed_phrase()
            .map_err(CoreXError::from)
            .map(SeedPhrase::from)?;

        let inner_identity =
            crate::try_identity_from_seed(seed.as_ref()).map_err(CoreXError::from)?;

        Ok(MasterIdentity::<W>::new(inner_identity))
    }

    pub fn new(inner_identity: Identity) -> Self {
        let wallet = W::new().unwrap();

        Self {
            inner_identity,
            wallet,
        }
    }
}

impl<W: WalletFactory + 'static> MasterIdentityApi for MasterIdentity<W> {
    fn get_seed_phrase(&self) -> SeedPhraseWords {
        self.inner_identity.get_seed_phrase()
    }

    fn get_signing_keypair(&self) -> Box<dyn CryptoSigningKeypair> {
        Box::new(self.inner_identity.signing_key())
    }

    fn create_wildland_identity(
        &self,
        identity_type: WildlandIdentityType,
        name: String,
    ) -> Result<Arc<Mutex<dyn WildlandIdentityApi>>, CoreXError> {
        let keypair = self.get_signing_keypair().into();
        let identity =
            WildlandIdentity::<W>::new(identity_type, keypair, name, self.wallet.clone());

        identity.save()?;

        Ok(Arc::new(Mutex::new(identity)))
    }
}

impl<W: WalletFactory> Display for MasterIdentity<W> {
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
