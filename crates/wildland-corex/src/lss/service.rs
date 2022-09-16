use super::{api::LocalSecureStorage, result::LssResult};
use crate::{ForestRetrievalError, WildlandIdentity, DEFAULT_FOREST_KEY};
#[cfg(test)]
use mockall::mock;
use wildland_crypto::identity::SigningKeypair;

#[derive(Clone)]
pub struct LssService {
    lss: &'static dyn LocalSecureStorage,
}

#[cfg(test)]
mock! {
    pub LssService {
        pub fn save(&self, wildland_identity: WildlandIdentity) -> LssResult<Option<Vec<u8>>> ;
        pub fn get_default_forest(&self) -> Result<Option<WildlandIdentity>, ForestRetrievalError>;
    }
    impl Clone for LssService {
        fn clone(&self) -> Self;
    }
}

impl LssService {
    pub fn new(lss: &'static dyn LocalSecureStorage) -> Self {
        tracing::debug!("created new instance");
        Self { lss }
    }

    #[tracing::instrument(level = "debug", skip(self, wildland_identity))]
    pub fn save(&self, wildland_identity: WildlandIdentity) -> LssResult<Option<Vec<u8>>> {
        self.lss.insert(
            wildland_identity.to_string(),
            wildland_identity.get_keypair_bytes(),
        )
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_default_forest(&self) -> Result<Option<WildlandIdentity>, ForestRetrievalError> {
        self.lss
            .get(DEFAULT_FOREST_KEY.to_string())
            .map_err(|e| e.into())
            .and_then(|default_forest_value| {
                default_forest_value.map_or(Ok(None), |default_forest_value| {
                    let signing_key = SigningKeypair::try_from(default_forest_value)
                        .map_err(ForestRetrievalError::KeypairParseError)?;
                    Ok(Some(WildlandIdentity::Forest(0, signing_key)))
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalSecureStorage, LssResult, LssService, DEFAULT_FOREST_KEY};
    use crate::test_utilities::{create_signing_keypair, create_wildland_forest_identity};
    use mockall::{mock, predicate::eq};

    mock! {
        #[derive(Debug)]
        TestLSS {}
        impl LocalSecureStorage for TestLSS {
            fn insert(&self, key: String, value: Vec<u8>) -> LssResult<Option<Vec<u8>>> {}
            fn get(&self, key: String) -> LssResult<Option<Vec<u8>>> {}
            fn contains_key(&self, key: String) -> LssResult<bool> {}
            fn keys(&self) -> LssResult<Vec<String>> {}
            fn remove(&self, key: String) -> LssResult<Option<Vec<u8>>> {}
            fn len(&self) -> LssResult<usize> {}
            fn is_empty(&self) -> LssResult<bool> {}
        }
    }

    #[test]
    fn should_save_forest_identity() {
        // given
        let keypair = create_signing_keypair();
        let wildland_identity = create_wildland_forest_identity();
        let mut lss_mock = MockTestLSS::new();
        lss_mock
            .expect_insert()
            .with(eq(String::from(DEFAULT_FOREST_KEY)), eq(keypair.to_bytes()))
            .return_once(|_key, _value| Ok(None));
        let lss_mock_static_ref: &'static MockTestLSS = unsafe { std::mem::transmute(&lss_mock) };
        let lss_service = LssService::new(lss_mock_static_ref);

        // when
        let result = lss_service.save(wildland_identity).unwrap();

        // then
        assert!(result.is_none())
    }

    #[test]
    fn should_get_default_forest() {
        // given
        let keypair = create_signing_keypair();
        let keypair_bytes = keypair.to_bytes();
        let wildland_identity = create_wildland_forest_identity();
        let mut lss_mock = MockTestLSS::new();
        lss_mock
            .expect_get()
            .with(eq(String::from(DEFAULT_FOREST_KEY)))
            .return_once(|_| Ok(Some(keypair_bytes)));
        let lss_mock_static_ref: &'static MockTestLSS = unsafe { std::mem::transmute(&lss_mock) };
        let lss_service = LssService::new(lss_mock_static_ref);

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
        let lss_mock_static_ref: &'static MockTestLSS = unsafe { std::mem::transmute(&lss_mock) };
        let lss_service = LssService::new(lss_mock_static_ref);

        // when
        let result = lss_service.get_default_forest().unwrap();

        // then
        assert!(result.is_none());
    }
}
