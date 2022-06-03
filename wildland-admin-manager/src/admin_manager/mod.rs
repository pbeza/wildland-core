mod identity;

use crate::api::{self, AdminManagerError, AdminManagerIdentity, EmailClient, SeedPhrase};
pub use identity::CryptoIdentity;
use std::sync::{Arc, Mutex};

pub struct AdminManager {
    // TODO do we want to store more than one master identity
    // TODO do we want to keep mappings between a master identity and a set of device identities
    master_identity: Option<AdminManagerIdentity>,
    email: Option<Email>,
    email_client: Arc<dyn EmailClient>,
}

pub enum Email {
    Unverified {
        mailbox_address: String,
        verification_code: Option<String>,
    },
    Verified(String),
}

impl AdminManager {
    pub fn new(email_client: Arc<dyn EmailClient>) -> Self {
        Self {
            master_identity: None,
            email: None,
            email_client,
        }
    }
}

pub fn create_seed_phrase() -> api::AdminManagerResult<SeedPhrase> {
    wildland_corex::generate_random_seed_phrase()
        .map_err(AdminManagerError::from)
        .map(SeedPhrase::from)
}

impl api::AdminManager for AdminManager {
    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: &SeedPhrase,
    ) -> api::AdminManagerResult<AdminManagerIdentity> {
        let identity = CryptoIdentity::new(
            api::IdentityType::Master,
            name,
            wildland_corex::try_identity_from_seed(seed.as_ref())?,
        );
        self.master_identity = Some(Arc::new(Mutex::new(identity))); // TODO Can user have multiple master identities? If not should it be overwritten?
        Ok(self.master_identity.as_ref().unwrap().clone())
    }

    fn get_master_identity(&self) -> Option<AdminManagerIdentity> {
        self.master_identity.clone()
    }

    fn set_email(&mut self, email: String) {
        self.email = Some(Email::Unverified {
            mailbox_address: email,
            verification_code: None,
        });
    }

    fn send_verification_code(&mut self) -> api::AdminManagerResult<()> {
        match &mut self.email {
            Some(Email::Unverified {
                mailbox_address,
                verification_code,
            }) => {
                // TODO generate code
                let new_code = "123456";
                *verification_code = Some(new_code.to_owned());
                self.email_client.send(mailbox_address, new_code)
            }
            Some(Email::Verified(_)) => Err(AdminManagerError::EmailAlreadyVerified),
            None => Err(AdminManagerError::EmailCandidateNotSet),
        }
    }

    fn verify_email(&mut self, input_verification_code: String) -> api::AdminManagerResult<()> {
        match self
            .email
            .as_ref()
            .ok_or(AdminManagerError::EmailCandidateNotSet)?
        {
            Email::Unverified {
                mailbox_address: email,
                verification_code: Some(stored_verification_code),
            } => {
                if stored_verification_code == &input_verification_code {
                    self.email = Some(Email::Verified(email.clone()));
                    Ok(())
                } else {
                    Err(AdminManagerError::ValidationCodesDoNotMatch)
                }
            }
            Email::Unverified {
                verification_code: None,
                ..
            } => Err(AdminManagerError::VerificationCodeNotSent),
            Email::Verified(_) => Err(AdminManagerError::EmailAlreadyVerified),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{AdminManager as AdminManagerApi, MockEmailClient};

    #[test]
    fn cannot_verify_email_when_not_set() {
        let mut am = AdminManager::new(Arc::new(MockEmailClient::new()));
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailCandidateNotSet
        );
    }

    #[test]
    fn verification_fails_when_codes_do_not_match() {
        let mut email_client = MockEmailClient::new();
        email_client
            .expect_send()
            .times(1)
            .withf(|address, code| address == "email@email.com" && code == "123456")
            .returning(|_, _| Ok(()));

        let mut am = AdminManager::new(Arc::new(email_client));
        am.set_email("email@email.com".to_string());
        am.send_verification_code().unwrap();

        assert_eq!(
            am.verify_email("123455".to_owned()).unwrap_err(),
            AdminManagerError::ValidationCodesDoNotMatch
        );
    }

    #[test]
    fn verification_fails_if_email_is_already_verified() {
        let mut email_client = MockEmailClient::new();
        email_client
            .expect_send()
            .times(1)
            .withf(|address, code| address == "email@email.com" && code == "123456")
            .returning(|_, _| Ok(()));

        let mut am = AdminManager::new(Arc::new(email_client));
        am.set_email("email@email.com".to_string());
        am.send_verification_code().unwrap();
        assert!(am.verify_email("123456".to_owned()).is_ok());
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailAlreadyVerified
        );
    }
}
