use mockall::mock;
use wildland_corex::{ForestRetrievalError, LssResult, WildlandIdentity};

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
