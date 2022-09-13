use crate::{CoreXError, CorexResult, MasterIdentity};
use mockall_double::double;
use std::rc::Rc;
use wildland_crypto::identity::{self, Identity, MnemonicPhrase};

#[double]
use crate::LSSService;

pub fn generate_random_mnemonic() -> CorexResult<MnemonicPhrase> {
    identity::generate_random_mnemonic().map_err(CoreXError::from)
}

pub enum CreateUserInput {
    Mnemonic(Box<MnemonicPhrase>),
    Entropy(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct UserService {
    lss_service: Rc<LSSService>,
}

impl UserService {
    #[tracing::instrument(level = "debug", ret)]
    pub fn new(lss_service: Rc<LSSService>) -> Self {
        Self { lss_service }
    }

    #[tracing::instrument(level = "debug", skip(input, self))]
    pub fn create_user(&self, input: CreateUserInput, device_name: String) -> CorexResult<()> {
        if self.user_exists()? {
            return Err(CoreXError::UserAlreadyExists);
        }
        let crypto_identity = match input {
            CreateUserInput::Mnemonic(mnemonic) => {
                let mnemonic = *mnemonic;
                Identity::try_from(&mnemonic)?
            }
            CreateUserInput::Entropy(entropy) => Identity::try_from(entropy.as_slice())?,
        };
        let master_identity = MasterIdentity::new(Some(crypto_identity));
        let default_forest_identity = master_identity.create_forest_identity(0)?;
        let device_identity = master_identity.create_device_identity(device_name)?;

        self.lss_service.save(default_forest_identity)?;
        self.lss_service.save(device_identity)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn user_exists(&self) -> CorexResult<bool> {
        self.lss_service
            .get_default_forest()
            .map(|forest| forest.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::create_wildland_forest_identity;
    use crate::{WildlandIdentity, DEFAULT_FOREST_KEY};
    use hex_literal::hex;

    #[test]
    fn generated_mnemonic_has_proper_length() {
        let mnemonic = generate_random_mnemonic().unwrap();
        assert_eq!(mnemonic.len(), 12);
    }

    #[test]
    fn should_not_create_user_when_it_already_exists() {
        // given
        let forest_wildland_identity = create_wildland_forest_identity();
        let mut lss_service_mock = LSSService::default();
        lss_service_mock
            .expect_get_default_forest()
            .return_once(|| Ok(Some(forest_wildland_identity)));
        let user_service = UserService::new(Rc::new(lss_service_mock));

        // when
        let result =
            user_service.create_user(CreateUserInput::Entropy(vec![]), "My Mac".to_string());

        // then
        assert_eq!(result.unwrap_err(), CoreXError::UserAlreadyExists);
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

        let mut lss_service_mock = LSSService::default();
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
        let user_service = UserService::new(Rc::new(lss_service_mock));

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

        let mut lss_service_mock = LSSService::default();
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
        let user_service = UserService::new(Rc::new(lss_service_mock));

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
        let mut lss_service_mock = LSSService::default();
        lss_service_mock
            .expect_get_default_forest()
            .return_once(|| Ok(Some(forest_wildland_identity)));
        let user_service = UserService::new(Rc::new(lss_service_mock));

        // when
        let result = user_service.user_exists();

        // then
        assert!(result.unwrap());
    }
}
