//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
    errors::{
        RetrievalError, RetrievalResult, SingleErrVariantResult, SingleVariantError,
        UserCreationError, UserRetrievalError,
    },
    user::{generate_random_mnemonic, CreateUserInput, UserService},
};
use wildland_corex::{CryptoError, MnemonicPhrase};

#[derive(Clone)]
pub struct MnemonicPayload(MnemonicPhrase);

impl MnemonicPayload {
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_string(&self) -> String {
        self.0.join(" ")
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_vec(&self) -> Vec<String> {
        self.0.clone().into()
    }
}

impl From<MnemonicPhrase> for MnemonicPayload {
    #[tracing::instrument(level = "debug")]
    fn from(mnemonic: MnemonicPhrase) -> Self {
        Self(mnemonic)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoUser {
    pub this_device: String,
    pub all_devices: Vec<String>,
}

impl CargoUser {
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_string(&self) -> String {
        let CargoUser {
            this_device,
            all_devices,
        } = &self;
        let all_devices_str = all_devices
            .iter()
            .map(|d| format!("    {d}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "
This device: {this_device}
All devices:
{all_devices_str}
"
        )
    }
}

/// User management API
///
/// [`CargoUser`] can be created with the following methods:
/// - [`UserApi::create_user_from_entropy`]
/// - [`UserApi::create_user_from_mnemonic`]
///
///  Creating a new user means:
/// - checking if one does not exist yet
/// - generating forest identity
/// - generating device identity
/// - saving forest in CatLib
/// - saving forest uuid (CatLib key) in LSS
/// - saving forest and device identities (keypairs) in LSS
///
#[derive(Clone)]
pub struct UserApi {
    user_service: UserService,
}

impl UserApi {
    pub(crate) fn new(user_service: UserService) -> Self {
        Self { user_service }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn generate_mnemonic(&self) -> SingleErrVariantResult<MnemonicPayload, CryptoError> {
        tracing::trace!("generating mnemonic");
        generate_random_mnemonic()
            .map_err(SingleVariantError::Failure)
            .map(MnemonicPayload::from)
    }

    /// Creates [`MnemonicPayload`] basing on a vector of words. The result may be used for creation
    /// User with [`UserApi::create_user_from_mnemonic`].
    ///
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn create_mnemonic_from_vec(
        &self,
        words: Vec<String>,
    ) -> SingleErrVariantResult<MnemonicPayload, String> {
        tracing::trace!("creating mnemonic from vec");
        Ok(MnemonicPayload(MnemonicPhrase::try_from(words).map_err(
            |_| SingleVariantError::Failure("Invalid mnemonic words".to_owned()),
        )?))
    }

    #[tracing::instrument(level = "debug", skip(self, entropy))]
    pub fn create_user_from_entropy(
        &self,
        entropy: Vec<u8>,
        device_name: String,
    ) -> SingleErrVariantResult<CargoUser, UserCreationError> {
        tracing::debug!("creating new user");
        self.user_service
            .create_user(CreateUserInput::Entropy(entropy), device_name)
            .map_err(SingleVariantError::Failure)
    }

    #[tracing::instrument(level = "debug", skip(self, mnemonic))]
    pub fn create_user_from_mnemonic(
        &self,
        mnemonic: &MnemonicPayload,
        device_name: String,
    ) -> SingleErrVariantResult<CargoUser, UserCreationError> {
        tracing::debug!("creating new user");
        self.user_service
            .create_user(
                CreateUserInput::Mnemonic(Box::new(mnemonic.0.clone())),
                device_name,
            )
            .map_err(SingleVariantError::Failure)
    }

    /// Gets user if it exists
    ///
    pub fn get_user(&self) -> RetrievalResult<CargoUser, UserRetrievalError> {
        tracing::debug!("getting user");
        let user = self.user_service.get_user().map_err(|e| match e {
            UserRetrievalError::ForestRetrievalError(_)
            | UserRetrievalError::LssError(_)
            | UserRetrievalError::CatlibError(_)
            | UserRetrievalError::DeviceMetadataNotFound => RetrievalError::Unexpected(e),
            UserRetrievalError::ForestNotFound(e) => RetrievalError::NotFound(e),
        })?;
        match user {
            Some(user) => Ok(user),
            None => Err(RetrievalError::NotFound("User not found.".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    use wildland_corex::{LocalSecureStorage, LssResult, LssService};

    use crate::{errors::RetrievalError, user::UserService};

    use super::{CargoUser, UserApi};

    #[derive(Default)]
    struct LssStub {
        storage: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl LocalSecureStorage for LssStub {
        fn insert(&self, key: String, value: Vec<u8>) -> LssResult<Option<Vec<u8>>> {
            Ok(self.storage.borrow_mut().insert(key, value))
        }

        fn get(&self, key: String) -> LssResult<Option<Vec<u8>>> {
            Ok(self.storage.try_borrow().unwrap().get(&key).cloned())
        }

        fn contains_key(&self, key: String) -> LssResult<bool> {
            Ok(self.storage.borrow().contains_key(&key))
        }

        fn keys(&self) -> LssResult<Vec<String>> {
            Ok(self.storage.borrow().keys().cloned().collect())
        }

        fn remove(&self, key: String) -> LssResult<Option<Vec<u8>>> {
            Ok(self.storage.borrow_mut().remove(&key))
        }

        fn len(&self) -> LssResult<usize> {
            Ok(self.storage.borrow().len())
        }

        fn is_empty(&self) -> LssResult<bool> {
            Ok(self.storage.borrow().is_empty())
        }
    }

    #[test]
    fn get_user_should_return_none_if_it_does_not_exist() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let lss_service = LssService::new(lss_ref);
        let user_service = UserService::new(lss_service);
        let user_api = UserApi::new(user_service);

        let user_result = user_api.get_user();
        assert_eq!(
            user_result.unwrap_err(),
            RetrievalError::NotFound("Forest identity keypair not found".to_owned())
        )
    }

    #[test]
    fn create_user_should_return_user_structure() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let lss_service = LssService::new(lss_ref);
        let user_service = UserService::new(lss_service);
        let user_api = UserApi::new(user_service);

        let mnemonic = user_api.generate_mnemonic().unwrap();
        let device_name = "device name".to_string();
        let user = user_api
            .create_user_from_mnemonic(&mnemonic, device_name.clone())
            .unwrap();

        let expected_user = CargoUser {
            this_device: device_name.clone(),
            all_devices: vec![device_name],
        };
        assert_eq!(user, expected_user);
    }

    #[test]
    fn get_user_should_return_some_if_it_was_created() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let lss_service = LssService::new(lss_ref);
        let user_service = UserService::new(lss_service);
        let user_api = UserApi::new(user_service);

        let mnemonic = user_api.generate_mnemonic().unwrap();
        let device_name = "device name".to_string();
        let _ = user_api
            .create_user_from_mnemonic(&mnemonic, device_name.clone())
            .unwrap();

        let user = user_api.get_user().unwrap();
        let expected_user = CargoUser {
            this_device: device_name.clone(),
            all_devices: vec![device_name],
        };
        assert_eq!(user, expected_user);
    }
}
