use wildland_crypto::identity::{new_device_identity, Identity as CryptoIdentity};

use crate::{CoreXError, CorexResult};

use super::wildland::WildlandIdentity;

#[derive(Debug)]
pub struct MasterIdentity {
    crypto_identity: Option<CryptoIdentity>,
}

impl MasterIdentity {
    #[tracing::instrument(level = "debug", ret)]
    pub fn new(crypto_identity: Option<CryptoIdentity>) -> Self {
        Self { crypto_identity }
    }

    #[tracing::instrument(level = "debug", ret)]
    pub fn create_forest_identity(&self, index: u64) -> CorexResult<WildlandIdentity> {
        let keypair = self
            .crypto_identity
            .as_ref()
            .map(|identity| identity.forest_keypair(index))
            .ok_or_else(|| {
                CoreXError::CannotCreateForestIdentityError(
                    "Crypto identity is required to create a new forest".to_string(),
                )
            })?;
        let identity = WildlandIdentity::Forest(index, keypair);

        Ok(identity)
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn create_device_identity(&self, name: String) -> CorexResult<WildlandIdentity> {
        let keypair = new_device_identity();
        let identity = WildlandIdentity::Device(name, keypair);

        Ok(identity)
    }
}

#[cfg(test)]
mod tests {
    use crate::{generate_random_mnemonic, CoreXError, MasterIdentity, WildlandIdentity};
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
            CoreXError::CannotCreateForestIdentityError(
                "Crypto identity is required to create a new forest".to_string()
            )
        );
    }

    #[test]
    fn should_create_device_identity_with_crypto_identity() {
        let crypto_identity = create_crypto_identity();
        let master_identity = MasterIdentity::new(Some(crypto_identity));
        let device_identity = master_identity
            .create_device_identity("Device 1".to_string())
            .unwrap();

        assert!(matches!(device_identity, WildlandIdentity::Device(_, _)));
        assert_eq!(device_identity.get_identifier(), "Device 1");
        assert!(!device_identity.get_private_key().is_empty());
        assert!(!device_identity.get_public_key().is_empty());
    }

    #[test]
    fn should_create_device_identity_without_crypto_identity() {
        let master_identity = MasterIdentity::new(None);
        let device_identity = master_identity
            .create_device_identity("Device 1".to_string())
            .unwrap();

        assert!(matches!(device_identity, WildlandIdentity::Device(_, _)));
        assert_eq!(device_identity.get_identifier(), "Device 1");
        assert!(!device_identity.get_private_key().is_empty());
        assert!(!device_identity.get_public_key().is_empty());
    }
}
