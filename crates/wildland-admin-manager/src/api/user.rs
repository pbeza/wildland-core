use crate::{AdminManagerError, AdminManagerResult};
use wildland_corex::{generate_random_seed_phrase, SeedPhrase};

#[derive(Default)]
pub struct UserApi;

impl UserApi {
    pub fn generate_mnemonic(&self) -> AdminManagerResult<SeedPhrase> {
        generate_random_seed_phrase()
            .map_err(AdminManagerError::from)
            .map(SeedPhrase::from)
    }

    pub fn create_user_from_entropy(&self, entropy: Vec<u8>) {
        // TODO
    }
    pub fn create_user_from_mnemonic(&self, mnemonic: SeedPhrase) {
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
