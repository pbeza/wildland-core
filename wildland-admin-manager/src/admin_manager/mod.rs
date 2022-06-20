use std::fmt::Debug;

use crate::api::{
    self, AdminManagerApi, AdminManagerError, AdminManagerResult, MasterIdentityApi, SeedPhrase,
    WildlandIdentityType,
};
pub use api::{MasterIdentity, WildlandIdentity};
use wildland_corex::{file_wallet_factory, WalletFactoryType};

#[derive(Clone, Debug, Default)]
pub struct AdminManager {
    email: Option<Email>,
}

const WALLET_FACTORY: WalletFactoryType = &file_wallet_factory;

#[derive(Debug, Clone)]
pub enum Email {
    Unverified {
        mailbox_address: String,
        verification_code: String,
    },
    Verified(String),
}

impl AdminManager {
    fn create_forest_identity(
        &self,
        master_identity: Box<dyn MasterIdentityApi>,
        name: String,
    ) -> AdminManagerResult<api::WildlandIdentity> {
        let forest_id =
            master_identity.create_wildland_identity(WildlandIdentityType::Forest, name)?;

        Ok(forest_id)
    }

    fn create_device_identity(&self, name: String) -> AdminManagerResult<api::WildlandIdentity> {
        // TODO Control over wallet type should be realized by some method call or passing specific parameter rather than using Admin Manager type for it (generics)
        let master_identity = wildland_corex::MasterIdentity::new(WALLET_FACTORY)?;
        let device_id =
            master_identity.create_wildland_identity(WildlandIdentityType::Device, name)?;

        Ok(device_id)
    }
}

impl AdminManagerApi for AdminManager {
    fn create_seed_phrase() -> AdminManagerResult<SeedPhrase> {
        wildland_corex::generate_random_seed_phrase()
            .map_err(AdminManagerError::from)
            .map(SeedPhrase::from)
    }

    fn set_email(&mut self, email: String) {
        // TODO generate code
        let verification_code = "123456".to_owned();
        self.email = Some(Email::Unverified {
            mailbox_address: email,
            verification_code,
        });
    }

    fn send_verification_code(&mut self) -> AdminManagerResult<()> {
        // TODO actually send the code
        Ok(())
    }

    fn verify_email(&mut self, input_verification_code: String) -> AdminManagerResult<()> {
        match self
            .email
            .as_ref()
            .ok_or(AdminManagerError::EmailCandidateNotSet)?
        {
            Email::Unverified {
                mailbox_address: email,
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

    fn create_wildland_identities(
        &self,
        seed: &SeedPhrase,
        device_name: String,
    ) -> AdminManagerResult<api::IdentityPair> {
        let master_identity = wildland_corex::MasterIdentity::with_identity(
            wildland_corex::try_identity_from_seed(seed.as_ref())?,
            WALLET_FACTORY,
        );

        let forest_id = self.create_forest_identity(Box::new(master_identity), String::from(""))?;
        let device_id = self.create_device_identity(device_name)?;

        Ok(api::IdentityPair {
            forest_id,
            device_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use wildland_corex::FileWallet;

    use super::*;
    use crate::api::AdminManagerApi;

    #[test]
    fn cannot_verify_email_when_not_set() {
        let mut am = AdminManager::<FileWallet>::new().unwrap();
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailCandidateNotSet
        );
    }

    #[test]
    fn verification_fails_when_codes_do_not_match() {
        let mut am = AdminManager::<FileWallet>::new().unwrap();
        am.set_email("email@email.com".to_string());
        am.send_verification_code().unwrap();
        assert_eq!(
            am.verify_email("123455".to_owned()).unwrap_err(),
            AdminManagerError::ValidationCodesDoNotMatch
        );
    }

    #[test]
    fn verification_fails_if_email_is_already_verified() {
        let mut am = AdminManager::<FileWallet>::new().unwrap();
        am.set_email("email@email.com".to_string());
        am.send_verification_code().unwrap();
        assert!(am.verify_email("123456".to_owned()).is_ok());
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailAlreadyVerified
        );
    }
}
