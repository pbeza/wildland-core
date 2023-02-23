#[cfg(test)]
pub(crate) mod test {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use rstest::fixture;
    use uuid::Bytes;
    use wildland_catlib::CatLib;
    use wildland_corex::catlib_service::CatLibService;
    use wildland_corex::{LocalSecureStorage, LssResult};

    #[fixture]
    pub(crate) fn catlib_service() -> CatLibService {
        let uuid = uuid::Builder::from_random_bytes(rand::random::<Bytes>()).into_uuid();
        let redis_url =
            std::env::var("CARGO_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".into());
        let catlib = Rc::new(CatLib::new(redis_url, Some(uuid.to_string())));
        CatLibService::new(catlib)
    }

    #[fixture]
    pub(crate) fn lss_stub() -> &'static dyn LocalSecureStorage {
        #[derive(Default)]
        struct LssStub {
            storage: RefCell<HashMap<String, String>>,
        }

        impl LocalSecureStorage for LssStub {
            fn insert(&self, key: String, value: String) -> LssResult<Option<String>> {
                Ok(self.storage.borrow_mut().insert(key, value))
            }

            fn get(&self, key: String) -> LssResult<Option<String>> {
                Ok(self.storage.try_borrow().unwrap().get(&key).cloned())
            }

            fn contains_key(&self, key: String) -> LssResult<bool> {
                Ok(self.storage.borrow().contains_key(&key))
            }

            fn keys(&self) -> LssResult<Vec<String>> {
                Ok(self.storage.borrow().keys().cloned().collect())
            }

            fn keys_starting_with(&self, prefix: String) -> LssResult<Vec<String>> {
                Ok(self
                    .storage
                    .borrow()
                    .keys()
                    .filter(|key| key.starts_with(&prefix))
                    .cloned()
                    .collect())
            }

            fn remove(&self, key: String) -> LssResult<Option<String>> {
                Ok(self.storage.borrow_mut().remove(&key))
            }

            fn len(&self) -> LssResult<usize> {
                Ok(self.storage.borrow().len())
            }

            fn is_empty(&self) -> LssResult<bool> {
                Ok(self.storage.borrow().is_empty())
            }
        }

        Box::leak(Box::<LssStub>::default())
    }
}
