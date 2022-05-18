mod identity;

use api::AdminManagerError;
pub use identity::Identity;
use wildland_admin_manager_api as api;
use wildland_crypto::identity as crypto_identity;

pub enum Email {
    Unverified {
        email: String,
        verification_code: String,
    },
    Verified(String),
}

pub struct AdminManager<I: api::Identity> {
    // TODO do we want to store more than one master identity
    // TODO do we want to keep mappings between a master identity and a set of device identities
    master_identity: Option<I>,
    email: Option<Email>,
}

impl Default for AdminManager<Identity> {
    fn default() -> Self {
        Self {
            master_identity: Default::default(),
            email: Default::default(),
        }
    }
}

impl api::AdminManager for AdminManager<Identity> {
    type Identity = Identity;

    fn create_master_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: api::SeedPhraseWords,
    ) -> api::AdminManagerResult<Identity> {
        let identity = Identity::new(
            api::IdentityType::Master,
            name,
            seed.try_into().map_err(AdminManagerError::from)?, // TODO delegate to corex ?
        );
        self.master_identity = Some(identity.clone()); // TODO Can user have multiple master identities? If not should it be overwritten?
        Ok(identity)
    }

    fn create_device_identity_from_seed_phrase(
        &mut self,
        name: String,
        seed: api::SeedPhraseWords,
    ) -> api::AdminManagerResult<Identity> {
        let identity = Identity::new(
            api::IdentityType::Device,
            name,
            seed.try_into().map_err(AdminManagerError::from)?, // TODO delegate to corex ?
        );
        // TODO keep it somehow?
        Ok(identity)
    }

    fn create_seed_phrase() -> api::AdminManagerResult<api::SeedPhraseWords> {
        // TODO delegate to corex ?
        crypto_identity::generate_random_seed_phrase().map_err(AdminManagerError::from)
    }

    fn get_master_identity(&self) -> Option<Identity> {
        self.master_identity.clone()
    }

    fn send_verification_code(&mut self, email: String) -> api::AdminManagerResult<()> {
        // TODO generate code
        let verification_code = "1232456".to_owned();
        // TODO actually send the code
        self.email = Some(Email::Unverified {
            email,
            verification_code,
        });
        Ok(())
    }

    fn verify_email(&mut self, input_verification_code: String) -> api::AdminManagerResult<()> {
        match self
            .email
            .as_ref()
            .ok_or(AdminManagerError::EmailCandidateNotSet)?
        {
            Email::Unverified {
                email,
                verification_code: stored_verification_code,
            } => {
                if stored_verification_code == &input_verification_code {
                    self.email = Some(Email::Verified(email.clone()));
                } else {
                    return Err(AdminManagerError::ValidationCodesDoNotMatch);
                }
            }
            Email::Verified(_) => return Err(AdminManagerError::EmailAlreadyVerified),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AdminManager;
    use wildland_admin_manager_api::{AdminManager as AdminManagerApi, AdminManagerError};

    #[test]
    fn cannot_verify_email_when_not_set() {
        let mut am = AdminManager::default();
        assert_eq!(
            am.verify_email("1232456".to_owned()).unwrap_err(),
            AdminManagerError::EmailCandidateNotSet
        );
    }

    #[test]
    fn verification_fails_when_codes_do_not_match() {
        let mut am = AdminManager::default();
        am.send_verification_code("email@email.com".to_string())
            .unwrap();
        assert_eq!(
            am.verify_email("1232455".to_owned()).unwrap_err(),
            AdminManagerError::ValidationCodesDoNotMatch
        );
    }

    #[test]
    fn verification_fails_if_email_is_already_verified() {
        let mut am = AdminManager::default();
        am.send_verification_code("email@email.com".to_string())
            .unwrap();
        assert!(am.verify_email("1232456".to_owned()).is_ok());
        assert_eq!(
            am.verify_email("1232456".to_owned()).unwrap_err(),
            AdminManagerError::EmailAlreadyVerified
        );
    }
}
