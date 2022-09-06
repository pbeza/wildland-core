use thiserror::Error;
use wildland_crypto::{
    error::KeyDeriveError,
    identity::{new_device_identity, Identity as CryptoIdentity},
};

use super::wildland::WildlandIdentity;

#[derive(Debug)]
pub struct MasterIdentity {
    crypto_identity: Option<CryptoIdentity>,
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ForestIdentityCreationError {
    #[error("Crypto identity is required to create a new forest")]
    CryptoIdentityNotFound,
    #[error(transparent)]
    KeyDeriveError(#[from] KeyDeriveError),
}

impl MasterIdentity {
    #[tracing::instrument(level = "debug", skip(crypto_identity))]
    pub fn new(crypto_identity: Option<CryptoIdentity>) -> Self {
        tracing::debug!("creating new identity");
        Self { crypto_identity }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn create_forest_identity(
        &self,
        index: u64,
    ) -> Result<WildlandIdentity, ForestIdentityCreationError> {
        let keypair = self
            .crypto_identity
            .as_ref()
            .map(|identity| identity.forest_keypair(index))
            .ok_or(ForestIdentityCreationError::CryptoIdentityNotFound)??;
        let identity = WildlandIdentity::Forest(index, keypair);

        Ok(identity)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn create_device_identity(&self, name: String) -> WildlandIdentity {
        let keypair = new_device_identity();
        WildlandIdentity::Device(name, keypair)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        generate_random_mnemonic, ForestIdentityCreationError, MasterIdentity, WildlandIdentity,
    };
    use wildland_crypto::identity::Identity;

    fn create_crypto_identity() -> Identity {
        generate_random_mnemonic()
            .map(|mnemonic| Identity::try_from(&mnemonic).unwrap())
            .unwrap()
    }

    #[test]
    fn should_create_forest_identity() {
        let crypto_identity = create_crypto_identity();
        let master_identity = MasterIdentity::new(Some(crypto_identity));
        let forest_identity = master_identity.create_forest_identity(0).unwrap();

        assert!(matches!(forest_identity, WildlandIdentity::Forest(_, _)));
        assert_eq!(forest_identity.get_identifier(), "0");
        assert!(!forest_identity.get_private_key().is_empty());
        assert!(!forest_identity.get_public_key().is_empty());
    }

    #[test]
    fn should_not_create_forest_identity_without_crypto_identity() {
        let master_identity = MasterIdentity::new(None);
        let result = master_identity.create_forest_identity(0);
        assert_eq!(
            result.unwrap_err(),
            ForestIdentityCreationError::CryptoIdentityNotFound
        );
    }

    #[test]
    fn should_create_device_identity_with_crypto_identity() {
        let crypto_identity = create_crypto_identity();
        let master_identity = MasterIdentity::new(Some(crypto_identity));
        let device_identity = master_identity.create_device_identity("Device 1".to_string());

        assert!(matches!(device_identity, WildlandIdentity::Device(_, _)));
        assert_eq!(device_identity.get_identifier(), "Device 1");
        assert!(!device_identity.get_private_key().is_empty());
        assert!(!device_identity.get_public_key().is_empty());
    }

    #[test]
    fn should_create_device_identity_without_crypto_identity() {
        let master_identity = MasterIdentity::new(None);
        let device_identity = master_identity.create_device_identity("Device 1".to_string());

        assert!(matches!(device_identity, WildlandIdentity::Device(_, _)));
        assert_eq!(device_identity.get_identifier(), "Device 1");
        assert!(!device_identity.get_private_key().is_empty());
        assert!(!device_identity.get_public_key().is_empty());
    }
}
