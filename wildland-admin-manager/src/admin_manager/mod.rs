use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::api::{
    self, AdminManagerApi, AdminManagerError, AdminManagerResult, MasterIdentityApi, SeedPhrase,
    WalletFactory, WildlandIdentityType, WildlandWallet,
};
pub use api::{MasterIdentity, WildlandIdentity};

#[derive(Clone)]
pub struct AdminManager<W: WalletFactory> {
    phantom: PhantomData<W>,
    email: Option<Email>,
    wallet: WildlandWallet,
}

#[derive(Debug, Clone)]
pub enum Email {
    Unverified {
        mailbox_address: String,
        verification_code: String,
    },
    Verified(String),
}

impl<W: WalletFactory + 'static> AdminManager<W> {
    pub fn new() -> AdminManagerResult<Self> {
        let wallet = W::new().unwrap();

        Ok(AdminManager {
            phantom: PhantomData,
            email: None,
            wallet: Arc::new(Mutex::new(wallet)),
        })
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
        let master_identity = wildland_corex::MasterIdentity::<W>::default()?;
        let forest_id =
            master_identity.create_wildland_identity(WildlandIdentityType::Device, name)?;

        Ok(forest_id)
    }
}

impl<W: WalletFactory + 'static> AdminManagerApi for AdminManager<W> {
    fn get_wallet(&self) -> AdminManagerResult<WildlandWallet> {
        Ok(self.wallet.clone())
    }

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
        let master_identity = wildland_corex::MasterIdentity::<W>::new(
            wildland_corex::try_identity_from_seed(seed.as_ref())?,
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
