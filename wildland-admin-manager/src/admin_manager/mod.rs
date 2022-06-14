mod identity;

use crate::api::{self, AdminManagerError, AdminManagerIdentity, SeedPhrase};
pub use identity::CryptoIdentity;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct AdminManager {
    // TODO do we want to store more than one master identity
    // TODO do we want to keep mappings between a master identity and a set of device identities
    master_identity: Option<AdminManagerIdentity>,
    email: Option<EmailAddress>,
}

enum EmailAddress {
    Unverified(String),
    Verified(String),
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

    fn create_seed_phrase() -> api::AdminManagerResult<SeedPhrase> {
        wildland_corex::generate_random_seed_phrase()
            .map_err(AdminManagerError::from)
            .map(SeedPhrase::from)
    }

    fn get_master_identity(&self) -> Option<AdminManagerIdentity> {
        self.master_identity.clone()
    }

    fn set_email(&mut self, email: String) {
        self.email = Some(EmailAddress::Unverified(email));
    }

    fn request_verification_email(&mut self) -> api::AdminManagerResult<()> {
        // TODO send http request
        Ok(())
    }

    fn verify_email(&mut self, _verification_code: String) -> api::AdminManagerResult<()> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::AdminManager as AdminManagerApi;

    #[test]
    fn cannot_verify_email_when_not_set() {
        let mut am = AdminManager::default();
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailCandidateNotSet
        );
    }

    #[test]
    fn verification_fails_if_email_is_already_verified() {
        let mut am = AdminManager::default();
        am.set_email("email@email.com".to_string());
        am.request_verification_email().unwrap();
        assert!(am.verify_email("123456".to_owned()).is_ok());
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailAlreadyVerified
        );
    }
}
