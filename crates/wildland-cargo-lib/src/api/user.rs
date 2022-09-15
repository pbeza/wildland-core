use crate::errors::{CreationError, CreationResult, RetrievalError, RetrievalResult};
use wildland_corex::{
    generate_random_mnemonic, CreateUserInput, CryptoError, ForestRetrievalError, MnemonicPhrase,
    UserCreationError, UserService,
};

#[derive(Debug, Clone)]
pub struct MnemonicPayload(MnemonicPhrase);

impl MnemonicPayload {
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_string(&self) -> String {
        self.0.join(" ")
    }

    #[tracing::instrument(level = "debug", ret)]
    pub fn get_vec(&self) -> Vec<String> {
        self.0.clone().into()
    }
}

impl From<MnemonicPhrase> for MnemonicPayload {
    #[tracing::instrument(level = "debug", ret)]
    fn from(mnemonic: MnemonicPhrase) -> Self {
        Self(mnemonic)
    }
}

#[derive(Clone, Debug)]
pub struct UserPayload;

impl UserPayload {
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_string(&self) -> String {
        "User Payload".to_string()
    }
}

#[derive(Clone)]
pub struct UserApi {
    user_service: UserService,
}

impl UserApi {
    pub fn new(user_service: UserService) -> Self {
        Self { user_service }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn generate_mnemonic(&self) -> CreationResult<MnemonicPayload, CryptoError> {
        tracing::trace!("generating mnemonic");
        generate_random_mnemonic()
            .map_err(CreationError::NotCreated)
            .map(MnemonicPayload::from)
    }

    #[tracing::instrument(level = "debug", skip(self, entropy))]
    pub fn create_user_from_entropy(
        &self,
        entropy: Vec<u8>,
        device_name: String,
    ) -> CreationResult<(), UserCreationError> {
        tracing::debug!("creating new user");
        self.user_service
            .create_user(CreateUserInput::Entropy(entropy), device_name)
            .map_err(CreationError::NotCreated)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn create_user_from_mnemonic(
        &self,
        mnemonic: &MnemonicPayload,
        device_name: String,
    ) -> CreationResult<(), UserCreationError> {
        tracing::debug!("creating new user");
        self.user_service
            .create_user(
                CreateUserInput::Mnemonic(Box::new(mnemonic.0.clone())),
                device_name,
            )
            .map_err(CreationError::NotCreated)
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_user(&self) -> RetrievalResult<UserPayload, ForestRetrievalError> {
        self.user_service
            .user_exists()
            .map_err(RetrievalError::Unexpected)
            .and_then(|exist| {
                if exist {
                    Ok(UserPayload)
                } else {
                    Err(RetrievalError::NotFound("User not found.".to_string()))
                }
            })
    }
}
