use crate::{AdminManagerError, AdminManagerResult};
use wildland_corex::{generate_random_mnemonic, MnemonicPhrase};

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

#[derive(Clone, Debug, Default)]
pub struct UserApi;

impl UserApi {
    pub fn generate_mnemonic(&self) -> AdminManagerResult<MnemonicPayload> {
        generate_random_mnemonic()
            .map_err(AdminManagerError::from)
            .map(MnemonicPayload::from)
    }

    pub fn create_user_from_entropy(&self, entropy: Vec<u8>) {
        // TODO
    }
    pub fn create_user_from_mnemonic(&self, mnemonic: MnemonicPayload) {
        // TODO
    }
    pub fn get_user(&self) {
        // TODO
    }
}

#[cfg(test)]
mod tests {
    use crate::api::user::UserApi;

    #[test]
    fn generated_mnemonic_has_proper_length() {
        let user_api = UserApi;
        let mnemonic = user_api.generate_mnemonic().unwrap();

        assert_eq!(mnemonic.get_vec().len(), 12);
    }
}
