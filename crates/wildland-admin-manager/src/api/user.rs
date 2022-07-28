use crate::{AdminManagerError, AdminManagerResult};
use wildland_corex::{generate_random_mnemonic, CreateUserInput, MnemonicPhrase, UserService};

#[derive(Debug, Clone)]
pub struct MnemonicPayload(MnemonicPhrase);

impl MnemonicPayload {
    pub fn get_string(&self) -> String {
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

#[derive(Clone, Debug)]
pub struct UserApi {
    user_service: UserService,
}

impl UserApi {
    pub fn new(user_service: UserService) -> Self {
        Self { user_service }
    }

    pub fn generate_mnemonic(&self) -> AdminManagerResult<MnemonicPayload> {
        generate_random_mnemonic()
            .map_err(AdminManagerError::from)
            .map(MnemonicPayload::from)
    }

    pub fn create_user_from_entropy(
        &self,
        entropy: Vec<u8>,
        device_name: String,
    ) -> AdminManagerResult<()> {
        self.user_service
            .create_user(CreateUserInput::Entropy(entropy), device_name)?;
        Ok(())
    }
    pub fn create_user_from_mnemonic(
        &self,
        mnemonic: &MnemonicPayload,
        device_name: String,
    ) -> AdminManagerResult<()> {
        self.user_service.create_user(
            CreateUserInput::Mnemonic(Box::new(mnemonic.0.clone())),
            device_name,
        )?;
        Ok(())
    }
    pub fn get_user(&self) {
        todo!()
    }
}
