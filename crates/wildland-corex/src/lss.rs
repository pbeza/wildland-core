use std::path::PathBuf;
use std::rc::Rc;

use wildland_local_secure_storage::{FileLSS, LocalSecureStorage};

use crate::{CoreXError, CorexResult, WildlandIdentity};

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
            wildland_identity.get_fingerprint(),
            wildland_identity.get_keypair_bytes(),
        )?;
        Ok(prev_value)
    }
}

#[cfg(test)]
mod tests {
    use crate::lss::LSSService;
    use crate::test_utilities::{SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY};
    use crate::{LocalSecureStorage, WildlandIdentity, WildlandIdentityType};
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

    #[test]
    fn should_save_forest_identity() {
        // given
        let keypair = SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap();
        let mut lss_mock = MockTestLSS::new();
        lss_mock
            .expect_insert()
            .with(
                eq(String::from("wildland.Forest.0")),
                eq(keypair.to_bytes()),
            )
            .return_once(|_key, _value| Ok(None));
        let wildland_identity =
            WildlandIdentity::new(WildlandIdentityType::Forest, keypair, "0".to_string());
        let lss_service = LSSService::new(Rc::new(lss_mock));

        // when
        let result = lss_service.save(wildland_identity).unwrap();

        // then
        assert!(result.is_none())
    }
}
