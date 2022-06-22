use std::fmt::Debug;

use crate::api::{
    self, AdminManagerApi, AdminManagerError, AdminManagerResult, MasterIdentityApi, SeedPhrase,
    WildlandIdentityType,
};
pub use api::{MasterIdentity, WildlandIdentity};
use wildland_corex::WalletFactoryType;

#[derive(Clone)]
pub struct AdminManager {
    wallet_factory: WalletFactoryType,
    email: Option<EmailAddress>,
}

#[derive(Debug, Clone)]
enum EmailAddress {
    Unverified(String),
    Verified(String),
}

impl std::fmt::Debug for AdminManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminManager")
            .field("email", &self.email)
            .finish()
    }
}

impl AdminManager {
    pub fn with_wallet_factory(wallet_factory: WalletFactoryType) -> Self {
        Self {
            wallet_factory,
            email: Default::default(),
        }
    }

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
        let master_identity = wildland_corex::MasterIdentity::new(self.wallet_factory)?;
        let device_id =
            master_identity.create_wildland_identity(WildlandIdentityType::Device, name)?;

        Ok(device_id)
    }
}

impl AdminManagerApi for AdminManager {
    fn create_seed_phrase(&self) -> AdminManagerResult<SeedPhrase> {
        wildland_corex::generate_random_seed_phrase()
            .map_err(AdminManagerError::from)
            .map(SeedPhrase::from)
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

    fn create_wildland_identities(
        &self,
        seed: &SeedPhrase,
        device_name: String,
    ) -> AdminManagerResult<api::IdentityPair> {
        let master_identity = wildland_corex::MasterIdentity::with_identity(
            wildland_corex::try_identity_from_seed(seed.as_ref())?,
            self.wallet_factory,
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
    use wildland_corex::file_wallet_factory;

    use super::*;
    use crate::api::AdminManagerApi;

    #[test]
    fn cannot_verify_email_when_not_set() {
        let mut am = AdminManager::with_wallet_factory(&file_wallet_factory);
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailCandidateNotSet
        );
    }

    #[test]
    fn verification_fails_if_email_is_already_verified() {
        let mut am = AdminManager::with_wallet_factory(&file_wallet_factory);
        am.set_email("email@email.com".to_string());
        am.request_verification_email().unwrap();
        assert!(am.verify_email("123456".to_owned()).is_ok());
        assert_eq!(
            am.verify_email("123456".to_owned()).unwrap_err(),
            AdminManagerError::EmailAlreadyVerified
        );
    }
}
