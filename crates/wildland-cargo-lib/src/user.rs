use crate::errors::UserCreationError;
use wildland_corex::{CryptoError, ForestRetrievalError, Identity, MasterIdentity, MnemonicPhrase};

#[cfg(test)]
use crate::test_utils::MockLssService as LssService;
#[cfg(not(test))]
use wildland_corex::LssService;

pub fn generate_random_mnemonic() -> Result<MnemonicPhrase, CryptoError> {
    wildland_corex::generate_random_mnemonic()
}

pub enum CreateUserInput {
    Mnemonic(Box<MnemonicPhrase>),
    Entropy(Vec<u8>),
}

#[derive(Clone)]
pub struct UserService {
    lss_service: LssService,
}

impl UserService {
    pub fn new(lss_service: LssService) -> Self {
        Self { lss_service }
    }

    #[tracing::instrument(level = "debug", skip(input, self))]
    pub fn create_user(
        &self,
        input: CreateUserInput,
        device_name: String,
    ) -> Result<(), UserCreationError> {
        log::trace!("Checking whether user exists.");
        if self
            .user_exists()
            .map_err(UserCreationError::ForestRetrievalError)?
        {
            return Err(UserCreationError::UserAlreadyExists);
        }
        log::trace!("User does not exist yet");
        let crypto_identity = match input {
            CreateUserInput::Mnemonic(mnemonic) => Identity::try_from(mnemonic.as_ref())?,
            CreateUserInput::Entropy(entropy) => Identity::try_from(entropy.as_slice())?,
        };
        let master_identity = MasterIdentity::new(Some(crypto_identity));
        let default_forest_identity = master_identity
            .create_forest_identity(0)
            .map_err(UserCreationError::ForestIdentityCreationError)?;
        let device_identity = master_identity.create_device_identity(device_name);

        self.lss_service.save(default_forest_identity)?;
        self.lss_service.save(device_identity)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn user_exists(&self) -> Result<bool, ForestRetrievalError> {
        self.lss_service
            .get_default_forest()
            .map(|forest| forest.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use wildland_corex::{SigningKeypair, WildlandIdentity, DEFAULT_FOREST_KEY};

    pub static SIGNING_PUBLIC_KEY: &str =
        "1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f";
    pub static SIGNING_SECRET_KEY: &str =
        "e02cdfa23ad7d94508108ad41410e556c5b0737e9c264d4a2304a7a45894fc57";

    pub fn create_signing_keypair() -> SigningKeypair {
        SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap()
    }

    pub fn create_wildland_forest_identity() -> WildlandIdentity {
        WildlandIdentity::Forest(0, create_signing_keypair())
    }

    #[test]
    fn generated_mnemonic_has_proper_length() {
        let mnemonic = generate_random_mnemonic().unwrap();
        assert_eq!(mnemonic.len(), 12);
    }

    #[test]
    fn should_not_create_user_when_it_already_exists() {
        // given
        let forest_wildland_identity = create_wildland_forest_identity();
        let mut lss_service_mock = LssService::default();
        lss_service_mock
            .expect_get_default_forest()
            .return_once(|| Ok(Some(forest_wildland_identity)));
        let user_service = UserService::new(lss_service_mock);

        // when
        let result =
            user_service.create_user(CreateUserInput::Entropy(vec![]), "My Mac".to_string());

        // then
        assert_eq!(result.unwrap_err(), UserCreationError::UserAlreadyExists);
    }

    #[test]
    fn should_create_user_from_entropy() {
        // given
        let entropy = hex!(
            "
            65426aa1176159d1929caea10514cddd
            d11235741001f125922f258a58716b58
            da63e3060fe461fe37e4ed201d76b132
            e35830929b0f4764e577d3da09ecb6d2
            12
        "
        );

        let mut lss_service_mock = LssService::default();
        lss_service_mock
            .expect_get_default_forest()
            .return_once(|| Ok(None));
        lss_service_mock
            .expect_save()
            .withf(|wildland_identity: &WildlandIdentity| {
                wildland_identity.to_string() == DEFAULT_FOREST_KEY
            })
            .returning(|_| Ok(None));
        lss_service_mock
            .expect_save()
            .withf(|wildland_identity: &WildlandIdentity| {
                wildland_identity.to_string() == "wildland.device.My Mac"
            })
            .returning(|_| Ok(None));
        let user_service = UserService::new(lss_service_mock);

        // when
        let result = user_service.create_user(
            CreateUserInput::Entropy(entropy.to_vec()),
            "My Mac".to_string(),
        );

        // then
        assert!(result.is_ok());
    }

    #[test]
    fn should_create_user_from_mnemonic() {
        // given
        let mnemonic: MnemonicPhrase = [
            "abandon".to_string(),
            "abandon".to_string(),
            "abandon".to_string(),
            "abandon".to_string(),
            "abandon".to_string(),
            "abandon".to_string(),
            "abandon".to_string(),
            "abandon".to_string(),
            "abandon".to_string(),
            "abandon".to_string(),
            "abandon".to_string(),
            "about".to_string(),
        ];

        let mut lss_service_mock = LssService::default();
        lss_service_mock
            .expect_get_default_forest()
            .return_once(|| Ok(None));
        lss_service_mock
            .expect_save()
            .withf(|wildland_identity: &WildlandIdentity| {
                wildland_identity.to_string() == DEFAULT_FOREST_KEY
            })
            .returning(|_| Ok(None));
        lss_service_mock
            .expect_save()
            .withf(|wildland_identity: &WildlandIdentity| {
                wildland_identity.to_string() == "wildland.device.My Mac"
            })
            .returning(|_| Ok(None));
        let user_service = UserService::new(lss_service_mock);

        // when
        let result = user_service.create_user(
            CreateUserInput::Mnemonic(Box::new(mnemonic)),
            "My Mac".to_string(),
        );

        // then
        assert!(result.is_ok());
    }

    #[test]
    fn should_return_true_if_user_exists() {
        // given
        let forest_wildland_identity = create_wildland_forest_identity();
        let mut lss_service_mock = LssService::default();
        lss_service_mock
            .expect_get_default_forest()
            .return_once(|| Ok(Some(forest_wildland_identity)));
        let user_service = UserService::new(lss_service_mock);

        // when
        let result = user_service.user_exists();

        // then
        assert!(result.unwrap());
    }
}
