use super::{api::LocalSecureStorage, result::LssResult};
use crate::{
    storage::{StorageTemplate, StorageTemplateType},
    ForestRetrievalError, LssError, WildlandIdentity, DEFAULT_FOREST_KEY,
};
use uuid::Uuid;
use wildland_crypto::identity::SigningKeypair;

const STORAGE_TEMPLATE_PREFIX: &str = "wildland.storage_template.";
const FOUNDATION_STORAGE_TEMPLATE_UUID_KEY: &str =
    "wildland.storage_template.foundation_storage_template_uuid";
#[derive(Clone)]
pub struct LssService {
    lss: &'static dyn LocalSecureStorage,
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
        tracing::trace!("Getting default forest.");
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

    #[tracing::instrument(level = "debug", skip(self, storage_template))]
    pub fn save_storage_template(
        &self,
        storage_template: &StorageTemplate,
    ) -> LssResult<Option<Vec<u8>>> {
        tracing::trace!("Saving storage template");
        if storage_template.storage_template_type() == StorageTemplateType::FoundationStorage {
            self.lss.insert(
                FOUNDATION_STORAGE_TEMPLATE_UUID_KEY.to_owned(),
                storage_template.uuid().as_bytes().to_vec(),
            )?;
        };
        self.lss.insert(
            format!("{STORAGE_TEMPLATE_PREFIX}{}", storage_template.uuid()),
            storage_template.data(),
        )
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_foundation_storage_template(&self) -> LssResult<Option<StorageTemplate>> {
        tracing::trace!("Getting foundation storage template.");
        let template_uuid_bytes_opt = self
            .lss
            .get(FOUNDATION_STORAGE_TEMPLATE_UUID_KEY.to_string())?;

        if let Some(uuid_bytes) = template_uuid_bytes_opt {
            let uuid = Uuid::from_slice(&uuid_bytes)
                .map_err(|e| LssError(format!("Could not create uuid out of bytes: {e}")))?;
            let template_bytes_opt = self.lss.get(format!("{STORAGE_TEMPLATE_PREFIX}{}", uuid))?;

            if let Some(template_bytes) = template_bytes_opt {
                Ok(Some(
                    StorageTemplate::try_from_bytes(
                        &template_bytes,
                        StorageTemplateType::FoundationStorage,
                    )
                    .map_err(|e| LssError(format!("Error while parsing storage template: {e}")))?,
                ))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
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
