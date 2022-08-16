use crate::{CargoLibError, CargoLibResult};
use wildland_corex::{generate_random_mnemonic, CreateUserInput, MnemonicPhrase, UserService};

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

#[derive(Clone, Debug)]
pub struct UserApi {
    user_service: UserService,
}

impl UserApi {
    #[tracing::instrument(level = "debug", ret)]
    pub fn new(user_service: UserService) -> Self {
        tracing::trace!("initialized subscriber");
        tracing::trace!("creating UserService");
        Self { user_service }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn generate_mnemonic(&self) -> CargoLibResult<MnemonicPayload> {
        tracing::trace!("generating mnemonic");
        generate_random_mnemonic()
            .map_err(CargoLibError::from)
            .map(MnemonicPayload::from)
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn create_user_from_entropy(
        &self,
        entropy: Vec<u8>,
        device_name: String,
    ) -> CargoLibResult<()> {
        self.user_service
            .create_user(CreateUserInput::Entropy(entropy), device_name)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn create_user_from_mnemonic(
        &self,
        mnemonic: &MnemonicPayload,
        device_name: String,
    ) -> CargoLibResult<()> {
        self.user_service.create_user(
            CreateUserInput::Mnemonic(Box::new(mnemonic.0.clone())),
            device_name,
        )?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_user(&self) -> CargoLibResult<Option<UserPayload>> {
        self.user_service
            .user_exists()
            .map(|exist| if exist { Some(UserPayload) } else { None })
            .map_err(CargoLibError::from)
    }
}
