use mockall::mock;
use wildland_corex::{
    storage::StorageTemplate, ForestRetrievalError, LocalSecureStorage, LssResult, WildlandIdentity,
};

#[cfg(test)]
mock! {
    pub LssService {
        pub fn new(lss: &'static dyn LocalSecureStorage) -> Self;
        pub fn save(&self, wildland_identity: WildlandIdentity) -> LssResult<Option<Vec<u8>>> ;
        pub fn get_default_forest(&self) -> Result<Option<WildlandIdentity>, ForestRetrievalError>;
        pub fn save_storage_template(
            &self,
            storage_template: &StorageTemplate,
        ) -> LssResult<Option<Vec<u8>>>;
    }
    impl Clone for LssService {
        fn clone(&self) -> Self;
    }
}
