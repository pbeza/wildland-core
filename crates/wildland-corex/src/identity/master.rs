use wildland_crypto::identity::{new_device_identity, Identity};

use crate::CoreXError;

use super::wildland::{WildlandIdentity, WildlandIdentityType};

pub struct MasterIdentity {
    crypto_identity: Option<Identity>,
}

impl MasterIdentity {
    pub fn new(crypto_identity: Option<Identity>) -> Self {
        Self { crypto_identity }
    }

    pub fn create_forest_identity(&self, index: u64) -> Result<WildlandIdentity, CoreXError> {
        let keypair = self
            .crypto_identity
            .as_ref()
            .map(|identity| identity.forest_keypair(index))
            .ok_or_else(|| {
                CoreXError::CannotCreateForestIdentityError(
                    "Crypto identity is required to create a new forest".to_string(),
                )
            })?;
        let identity =
            WildlandIdentity::new(WildlandIdentityType::Forest, keypair, index.to_string());

        Ok(identity)
    }

    pub fn create_device_identity(&self, name: String) -> Result<WildlandIdentity, CoreXError> {
        let keypair = new_device_identity();
        let identity = WildlandIdentity::new(WildlandIdentityType::Device, keypair, name);

        Ok(identity)
    }
}

#[cfg(test)]
mod tests {
    use crate::{generate_random_mnemonic, CoreXError, MasterIdentity, WildlandIdentityType};
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

        assert_eq!(forest_identity.get_type(), WildlandIdentityType::Forest);
        assert_eq!(forest_identity.get_name(), "0");
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
        let forest_identity = master_identity
            .create_device_identity("Device 1".to_string())
            .unwrap();

        assert_eq!(forest_identity.get_type(), WildlandIdentityType::Device);
        assert_eq!(forest_identity.get_name(), "Device 1");
        assert!(!forest_identity.get_private_key().is_empty());
        assert!(!forest_identity.get_public_key().is_empty());
    }

    #[test]
    fn should_create_device_identity_without_crypto_identity() {
        let master_identity = MasterIdentity::new(None);
        let forest_identity = master_identity
            .create_device_identity("Device 1".to_string())
            .unwrap();

        assert_eq!(forest_identity.get_type(), WildlandIdentityType::Device);
        assert_eq!(forest_identity.get_name(), "Device 1");
        assert!(!forest_identity.get_private_key().is_empty());
        assert!(!forest_identity.get_public_key().is_empty());
    }
}
