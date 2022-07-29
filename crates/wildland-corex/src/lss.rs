use std::path::PathBuf;
use std::rc::Rc;
use wildland_crypto::identity::SigningKeypair;

use wildland_local_secure_storage::{FileLSS, LocalSecureStorage};

use crate::{CoreXError, CorexResult, WildlandIdentity};

pub static DEFAULT_FOREST_KEY: &str = "wildland.forest.0";

pub fn create_file_lss(path: String) -> CorexResult<FileLSS> {
    FileLSS::new(PathBuf::from(path)).map_err(CoreXError::from)
}

struct LSSService {
    lss: Rc<dyn LocalSecureStorage>,
}

impl LSSService {
    pub fn new(lss: Rc<dyn LocalSecureStorage>) -> Self {
        Self { lss }
    }

    pub fn save(&self, wildland_identity: WildlandIdentity) -> CorexResult<Option<Vec<u8>>> {
        let prev_value = self.lss.insert(
            wildland_identity.to_string(),
            wildland_identity.get_keypair_bytes(),
        )?;
        Ok(prev_value)
    }

    pub fn get_default_forest(&self) -> CorexResult<Option<WildlandIdentity>> {
        let default_forest_value = self.lss.get(DEFAULT_FOREST_KEY.to_string())?;
        if default_forest_value.is_none() {
            return Ok(None);
        }
        let signing_key = SigningKeypair::try_from(default_forest_value.unwrap())?;
        Ok(Some(WildlandIdentity::Forest(0, signing_key)))
    }
}

#[cfg(test)]
mod tests {
    use crate::lss::LSSService;
    use crate::test_utilities::{SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY};
    use crate::{LocalSecureStorage, WildlandIdentity, DEFAULT_FOREST_KEY};
    use mockall::mock;
    use mockall::predicate::eq;
    use std::rc::Rc;
    use wildland_crypto::identity::SigningKeypair;
    use wildland_local_secure_storage::LSSResult;

    mock! {
        #[derive(Debug)]
        TestLSS {}
        impl LocalSecureStorage for TestLSS {
            fn insert(&self, key: String, value: Vec<u8>) -> LSSResult<Option<Vec<u8>>> {}
            fn get(&self, key: String) -> LSSResult<Option<Vec<u8>>> {}
            fn contains_key(&self, key: String) -> LSSResult<bool> {}
            fn keys(&self) -> LSSResult<Vec<String>> {}
            fn remove(&self, key: String) -> LSSResult<Option<Vec<u8>>> {}
            fn len(&self) -> LSSResult<usize> {}
            fn is_empty(&self) -> LSSResult<bool> {}
        }
    }

    fn signing_keypair() -> SigningKeypair {
        SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap()
    }

    fn wildland_forest_identity() -> WildlandIdentity {
        WildlandIdentity::Forest(0, signing_keypair())
    }

    #[test]
    fn should_save_forest_identity() {
        // given
        let keypair = signing_keypair();
        let wildland_identity = wildland_forest_identity();
        let mut lss_mock = MockTestLSS::new();
        lss_mock
            .expect_insert()
            .with(eq(String::from(DEFAULT_FOREST_KEY)), eq(keypair.to_bytes()))
            .return_once(|_key, _value| Ok(None));
        let lss_service = LSSService::new(Rc::new(lss_mock));

        // when
        let result = lss_service.save(wildland_identity).unwrap();

        // then
        assert!(result.is_none())
    }

    #[test]
    fn should_get_default_forest() {
        // given
        let keypair = signing_keypair();
        let keypair_bytes = keypair.to_bytes();
        let wildland_identity = wildland_forest_identity();
        let mut lss_mock = MockTestLSS::new();
        lss_mock
            .expect_get()
            .with(eq(String::from(DEFAULT_FOREST_KEY)))
            .return_once(|_| Ok(Some(keypair_bytes)));
        let lss_service = LSSService::new(Rc::new(lss_mock));

        // when
        let result = lss_service.get_default_forest().unwrap().unwrap();

        // then
        assert_eq!(result.to_string(), wildland_identity.to_string());
        assert_eq!(
            result.get_keypair_bytes(),
            wildland_identity.get_keypair_bytes()
        );
    }

    #[test]
    fn should_not_get_default_forest_when_it_does_not_exist() {
        // given
        let mut lss_mock = MockTestLSS::new();
        lss_mock
            .expect_get()
            .with(eq(String::from(DEFAULT_FOREST_KEY)))
            .return_once(|_| Ok(None));
        let lss_service = LSSService::new(Rc::new(lss_mock));

        // when
        let result = lss_service.get_default_forest().unwrap();

        // then
        assert!(result.is_none());
    }
}
