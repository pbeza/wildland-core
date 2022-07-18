use std::{fmt::Debug, rc::Rc};

use wildland_corex::{create_user, generate_random_mnemonic, CreateUserPayload, MnemonicPhrase, Wallet};

use crate::api::{AdminManagerApi, GenerateMnemonicResponse, UserApi};
use crate::{AdminManagerError, AdminManagerResult};

#[derive(Clone, Debug)]
pub struct AdminManager {
    wallet: Rc<dyn Wallet>,
    email: Option<EmailAddress>,
}

#[derive(Debug, Clone)]
enum EmailAddress {
    Unverified(String),
    Verified(String),
}

impl AdminManager {
    pub fn with_wallet(wallet: Box<dyn Wallet>) -> Self {
        Self {
            wallet: wallet.into(),
            email: Default::default(),
        }
    }
}

impl UserApi for AdminManager {
    fn generate_random_mnemonic(&self) -> AdminManagerResult<GenerateMnemonicResponse> {
        let mnemonic = generate_random_mnemonic()?;
        Ok(GenerateMnemonicResponse(mnemonic))
    }

    fn create_user_from_entropy(&self, entropy: &[u8]) -> AdminManagerResult<()> {
        create_user(CreateUserPayload::Entropy(entropy.to_vec()))?;
        Ok(())
    }

    fn create_user_from_mnemonic(&self, mnemonic: MnemonicPhrase) -> AdminManagerResult<()> {
        create_user(CreateUserPayload::Mnemonic(mnemonic))?;
        Ok(())
    }
}

impl AdminManagerApi for AdminManager {
    fn request_verification_email(&mut self) -> AdminManagerResult<()> {
        // TODO send http request
        Ok(())
    }

    fn set_email(&mut self, email: String) {
        self.email = Some(EmailAddress::Unverified(email));
    }

    fn verify_email(&mut self, _verification_code: String) -> AdminManagerResult<()> {
        match self
            .email
            .as_ref()
            .ok_or(AdminManagerError::EmailCandidateNotSet)?
        {
            EmailAddress::Unverified(email) => {
                let verified = true; // TODO send http request to verify email
                if verified {
                    self.email = Some(EmailAddress::Verified(email.clone()));
                } else {
                    return Err(AdminManagerError::ValidationCodesDoNotMatch);
                }
            }
            EmailAddress::Verified(_) => return Err(AdminManagerError::EmailAlreadyVerified),
        }

        Ok(())
    }

    fn list_secrets(&self) -> AdminManagerResult<Vec<wildland_corex::ManifestSigningKeypair>> {
        let ids = self
            .wallet
            .list_secrets()
            .map_err(AdminManagerError::Wallet)?;
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use wildland_corex::create_file_wallet;

    use crate::api::AdminManagerApi;

    use super::*;

    #[test]
    fn cannot_verify_email_when_not_set() {
        let mut am = AdminManager::with_wallet(create_file_wallet().unwrap());
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailCandidateNotSet
        );
    }

    #[test]
    fn verification_fails_if_email_is_already_verified() {
        let mut am = AdminManager::with_wallet(create_file_wallet().unwrap());
        am.set_email("email@email.com".to_string());
        am.request_verification_email().unwrap();
        assert!(am.verify_email("123456".to_owned()).is_ok());
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailAlreadyVerified
        );
    }
}
