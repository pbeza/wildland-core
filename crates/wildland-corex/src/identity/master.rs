use super::wildland::{WildlandIdentity, WildlandIdentityApi, WildlandIdentityType};
use crate::{crypto::SeedPhrase, CoreXError, CryptoSigningKeypair, WalletFactoryType};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};
use wildland_crypto::identity::{Identity, SeedPhraseWords};
use wildland_wallet::Wallet;

pub trait MasterIdentityApi: Display {
    fn get_seed_phrase(&self) -> SeedPhraseWords;
    fn get_signing_keypair(&self) -> Box<dyn CryptoSigningKeypair>;
    fn create_wildland_identity(
        &self,
        identity_type: WildlandIdentityType,
        name: String,
    ) -> Result<Arc<Mutex<dyn WildlandIdentityApi>>, CoreXError>;
}

type MasterIdentityWallet = Box<dyn Wallet>;
// #[derive(Clone)]
pub struct MasterIdentity {
    inner_identity: Identity,
    _wallet: MasterIdentityWallet, // TODO save it
    wallet_factory: WalletFactoryType,
}

impl MasterIdentity {
    pub fn new(wallet_factory: WalletFactoryType) -> Result<Self, CoreXError> {
        let seed = crate::generate_random_seed_phrase()
            .map_err(CoreXError::from)
            .map(SeedPhrase::from)?;

        let inner_identity =
            crate::try_identity_from_seed(seed.as_ref()).map_err(CoreXError::from)?;

        Ok(Self::with_identity(inner_identity, wallet_factory))
    }

    pub fn with_identity(inner_identity: Identity, wallet_factory: WalletFactoryType) -> Self {
        let wallet = wallet_factory().unwrap();
        Self {
            inner_identity,
            _wallet: wallet,
            wallet_factory,
        }
    }
}

impl MasterIdentityApi for MasterIdentity {
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
        let identity = WildlandIdentity::new(
            identity_type,
            keypair,
            name,
            (self.wallet_factory)().unwrap(),
        );

        identity.save()?;

        Ok(Arc::new(Mutex::new(identity)))
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
