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
    errors::{CreateMnemonicError, UserCreationError, UserRetrievalError},
    user::{generate_random_mnemonic, CreateUserInput, UserService},
};
use wildland_corex::{utils, MnemonicPhrase};

use super::cargo_user::CargoUser;

#[derive(Clone)]
pub struct MnemonicPayload(MnemonicPhrase);

/// Wrapper to check the mnemonic.
/// Accepts string. Returns Ok if the mnemonic is valid or Err otherwise
/// throws [`CryptoError`] if the mnemonic is invalid
pub fn check_phrase_mnemonic(phrase: &str) -> Result<(), CreateMnemonicError> {
    match utils::new_mnemonic_from_phrase(phrase) {
        Ok(_) => Ok(()),
        Err(_) => Err(CreateMnemonicError::InvalidMnemonicWords),
    }
}

impl MnemonicPayload {
    pub fn stringify(&self) -> String {
        self.0.join(" ")
    }

    pub fn get_vec(&self) -> Vec<String> {
        self.0.clone().into()
    }
}

impl From<MnemonicPhrase> for MnemonicPayload {
    fn from(mnemonic: MnemonicPhrase) -> Self {
        Self(mnemonic)
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

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn generate_mnemonic(&self) -> Result<MnemonicPayload, CreateMnemonicError> {
        tracing::trace!("generating mnemonic");
        generate_random_mnemonic()
            .map_err(|_| CreateMnemonicError::InvalidMnemonicWords)
            .map(MnemonicPayload::from)
    }

    /// Creates [`MnemonicPayload`] basing on a vector of words. The result may be used for creation
    /// User with [`UserApi::create_user_from_mnemonic`].
    ///
    /// It validates provided words
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_mnemonic_from_vec(
        &self,
        words: Vec<String>,
    ) -> Result<MnemonicPayload, CreateMnemonicError> {
        tracing::trace!("creating mnemonic from vec");
        check_phrase_mnemonic(words.join(" ").as_str())?;
        Ok(MnemonicPayload(
            MnemonicPhrase::try_from(words)
                .map_err(|_| CreateMnemonicError::InvalidMnemonicWords)?,
        ))
    }

    /// Creates [`MnemonicPayload`] basing on a space separated 12-word string. The result may be used for creation
    /// User with [`UserApi::create_user_from_mnemonic`].
    ///
    /// It validates provided words
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_mnemonic_from_string(
        &self,
        words: String,
    ) -> Result<MnemonicPayload, CreateMnemonicError> {
        tracing::trace!("creating mnemonic from vec");
        check_phrase_mnemonic(words.as_str())?;
        Ok(MnemonicPayload(
            MnemonicPhrase::try_from(words.split(" ").map(|w| w.to_owned()).collect::<Vec<_>>())
                .map_err(|_| CreateMnemonicError::InvalidMnemonicWords)?,
        ))
    }

    /// Creates user from entropy.
    ///
    /// Assumes high quality entropy of arbitrary length (>= 32 bytes) what is validated.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_user_from_entropy(
        &self,
        entropy: Vec<u8>,
        device_name: String,
    ) -> Result<CargoUser, UserCreationError> {
        tracing::debug!("creating new user");
        self.user_service
            .create_user(CreateUserInput::Entropy(entropy), device_name)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_user_from_mnemonic(
        &self,
        mnemonic: &MnemonicPayload,
        device_name: String,
    ) -> Result<CargoUser, UserCreationError> {
        tracing::debug!("creating new user");
        self.user_service.create_user(
            CreateUserInput::Mnemonic(Box::new(mnemonic.0.clone())),
            device_name,
        )
    }

    /// Gets user if it exists
    ///
    pub fn get_user(&self) -> Result<CargoUser, UserRetrievalError> {
        tracing::debug!("getting user");
        let user = self.user_service.get_user()?;
        match user {
            Some(user) => Ok(user),
            None => Err(UserRetrievalError::UserNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UserApi;
    use crate::api::config::FoundationStorageApiConfig;
    use crate::api::utils::test::{catlib_service, lss_stub};
    use crate::{errors::UserRetrievalError, user::UserService};
    use rstest::rstest;
    use wildland_corex::{CatLibService, LocalSecureStorage, LssService};

    #[rstest]
    fn create_mnemonic_from_string_with_valid_words_should_succeed(
        catlib_service: CatLibService,
        lss_stub: &'static dyn LocalSecureStorage,
    ) {
        let api = UserApi::new(UserService::new(
            LssService::new(lss_stub),
            catlib_service,
            FoundationStorageApiConfig::default(),
        ));
        let words = "wise exile kingdom cabbage improve also ridge fortune when joke market argue";

        assert!(api.create_mnemonic_from_string(words.to_owned()).is_ok());
    }

    #[rstest]
    fn create_mnemonic_from_string_with_invalid_words_should_return_err(
        catlib_service: CatLibService,
        lss_stub: &'static dyn LocalSecureStorage,
    ) {
        let api = UserApi::new(UserService::new(
            LssService::new(lss_stub),
            catlib_service,
            FoundationStorageApiConfig::default(),
        ));
        let words =
            "wise exile kingdom cabbage improve also ridge fortune when joke market invalid_word";

        assert!(api.create_mnemonic_from_string(words.to_owned()).is_err());
    }

    #[rstest]
    fn create_mnemonic_from_vec_with_valid_words_should_succeed(
        catlib_service: CatLibService,
        lss_stub: &'static dyn LocalSecureStorage,
    ) {
        let api = UserApi::new(UserService::new(
            LssService::new(lss_stub),
            catlib_service,
            FoundationStorageApiConfig::default(),
        ));
        let words = "wise exile kingdom cabbage improve also ridge fortune when joke market argue";

        assert!(api
            .create_mnemonic_from_vec(words.split(" ").map(ToOwned::to_owned).collect::<Vec<_>>())
            .is_ok());
    }

    #[rstest]
    fn create_mnemonic_from_vec_with_invalid_words_should_return_err(
        catlib_service: CatLibService,
        lss_stub: &'static dyn LocalSecureStorage,
    ) {
        let api = UserApi::new(UserService::new(
            LssService::new(lss_stub),
            catlib_service,
            FoundationStorageApiConfig::default(),
        ));
        let words =
            "wise exile kingdom cabbage improve also ridge fortune when joke market invalid_word";

        assert!(api
            .create_mnemonic_from_vec(words.split(" ").map(ToOwned::to_owned).collect::<Vec<_>>())
            .is_err());
    }

    #[rstest]
    fn get_user_should_return_none_if_it_does_not_exist(
        catlib_service: CatLibService,
        lss_stub: &'static dyn LocalSecureStorage,
    ) {
        let lss_service = LssService::new(lss_stub);
        let user_service = UserService::new(
            lss_service,
            catlib_service,
            FoundationStorageApiConfig::default(),
        );
        let user_api = UserApi::new(user_service);

        let user_result = user_api.get_user();
        assert_eq!(
            user_result.unwrap_err(),
            UserRetrievalError::ForestNotFound("Forest identity keypair not found".to_owned())
        )
    }

    #[rstest]
    fn create_user_should_return_user_structure(
        catlib_service: CatLibService,
        lss_stub: &'static dyn LocalSecureStorage,
    ) {
        let lss_service = LssService::new(lss_stub);
        let user_service = UserService::new(
            lss_service,
            catlib_service,
            FoundationStorageApiConfig::default(),
        );
        let user_api = UserApi::new(user_service);

        let mnemonic = user_api.generate_mnemonic().unwrap();
        let device_name = "device name".to_string();
        let user = user_api
            .create_user_from_mnemonic(&mnemonic, device_name.clone())
            .unwrap();

        assert_eq!(user.this_device(), device_name);
        assert_eq!(user.all_devices(), [device_name]);
    }

    #[rstest]
    fn get_user_should_return_some_if_it_was_created(
        catlib_service: CatLibService,
        lss_stub: &'static dyn LocalSecureStorage,
    ) {
        let lss_service = LssService::new(lss_stub);
        let user_service = UserService::new(
            lss_service,
            catlib_service,
            FoundationStorageApiConfig::default(),
        );
        let user_api = UserApi::new(user_service);

        let mnemonic = user_api.generate_mnemonic().unwrap();
        let device_name = "device name".to_string();
        let _ = user_api
            .create_user_from_mnemonic(&mnemonic, device_name.clone())
            .unwrap();

        let user = user_api.get_user().unwrap();
        assert_eq!(user.this_device(), device_name);
        assert_eq!(user.all_devices(), [device_name]);
    }
}
