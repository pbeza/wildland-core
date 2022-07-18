use std::{
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::api::{self, AdminManagerError, AdminManagerResult, SeedPhrase};
use wildland_corex::Wallet;
pub use wildland_corex::WildlandIdentity;

pub type WrappedWildlandIdentity = Arc<Mutex<WildlandIdentity>>;

#[derive(Clone, Debug)]
pub struct IdentityPair {
    pub forest_id: WrappedWildlandIdentity,
    pub device_id: WrappedWildlandIdentity,
}

impl IdentityPair {
    pub fn forest_id(&self) -> WrappedWildlandIdentity {
        self.forest_id.clone()
    }

    pub fn device_id(&self) -> WrappedWildlandIdentity {
        self.device_id.clone()
    }
}

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

    pub fn create_forest_identity(
        &self,
        seed: &SeedPhrase,
        name: String,
    ) -> AdminManagerResult<WrappedWildlandIdentity> {
        let master_identity = wildland_corex::MasterIdentity::with_identity(
            wildland_corex::try_identity_from_seed(seed.as_ref())?,
            self.wallet.clone(),
        );
        let forest_id =
            master_identity.create_wildland_identity(api::WildlandIdentityType::Forest, name)?;

        Ok(Arc::new(Mutex::new(forest_id)))
    }

    pub fn create_device_identity(
        &self,
        name: String,
    ) -> AdminManagerResult<WrappedWildlandIdentity> {
        let master_identity = wildland_corex::MasterIdentity::new(self.wallet.clone())?;
        let device_id =
            master_identity.create_wildland_identity(api::WildlandIdentityType::Device, name)?;

        Ok(Arc::new(Mutex::new(device_id)))
    }

    pub fn create_seed_phrase(&self) -> AdminManagerResult<SeedPhrase> {
        wildland_corex::generate_random_seed_phrase()
            .map_err(AdminManagerError::from)
            .map(SeedPhrase::from)
    }

    pub fn set_email(&mut self, email: String) {
        self.email = Some(EmailAddress::Unverified(email));
    }

    pub fn request_verification_email(&mut self) -> api::AdminManagerResult<()> {
        // TODO send http request
        Ok(())
    }

    pub fn verify_email(&mut self, _verification_code: String) -> api::AdminManagerResult<()> {
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

    pub fn create_wildland_identities(
        &self,
        seed: &SeedPhrase,
        device_name: String,
    ) -> AdminManagerResult<IdentityPair> {
        let forest_id = self.create_forest_identity(seed, String::from(""))?;
        let device_id = self.create_device_identity(device_name)?;

        Ok(IdentityPair {
            forest_id,
            device_id,
        })
    }

    pub fn list_secrets(&self) -> AdminManagerResult<Vec<wildland_corex::ManifestSigningKeypair>> {
        let ids = self
            .wallet
            .list_secrets()
            .map_err(AdminManagerError::Wallet)?;
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wildland_corex::create_file_wallet;

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
