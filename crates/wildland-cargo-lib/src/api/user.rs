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

#[derive(Clone, Debug)]
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
    /// User with [`create_user_from_mnemonic`].
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
    ) -> SingleErrVariantResult<(), UserCreationError> {
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
    ) -> SingleErrVariantResult<(), UserCreationError> {
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
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_user(&self) -> RetrievalResult<CargoUser, UserRetrievalError> {
        tracing::debug!("getting user");
        let user = self
            .user_service
            .get_user()
            .map_err(RetrievalError::Unexpected)?;
        match user {
            Some(user) => Ok(user),
            None => Err(RetrievalError::NotFound("User not found.".to_string())),
        }
    }
}
