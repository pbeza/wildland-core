//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt::{Debug, Display};

use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;
use wildland_crypto::identity::SigningKeypair;

use super::api::LocalSecureStorage;
use super::result::LssResult;
use crate::catlib_service::entities::{ForestManifest, Identity};
use crate::{ForestRetrievalError, LssError, WildlandIdentity, DEFAULT_FOREST_KEY};

#[derive(Clone)]
pub struct LssService {
    lss: &'static dyn LocalSecureStorage,
}

const THIS_DEVICE_KEYPAIR_KEY: &str = "wildland.device.keypair";
const THIS_DEVICE_NAME_KEY: &str = "wildland.device.name";

impl LssService {
    pub fn new(lss: &'static dyn LocalSecureStorage) -> Self {
        tracing::debug!("created new instance");
        Self { lss }
    }

    pub fn save_identity(&self, wildland_identity: &WildlandIdentity) -> LssResult<bool> {
        let key = match wildland_identity {
            WildlandIdentity::Forest(_, _) => wildland_identity.to_string(),
            WildlandIdentity::Device(device_name, _) => {
                self.serialize_and_save(THIS_DEVICE_NAME_KEY, &device_name)?;

                THIS_DEVICE_KEYPAIR_KEY.to_owned()
            }
        };
        self.serialize_and_save(key, &wildland_identity.get_keypair())
    }

    pub fn get_default_forest_identity(
        &self,
    ) -> Result<Option<WildlandIdentity>, ForestRetrievalError> {
        tracing::trace!("Getting default forest identity.");

        let optional_default_forest_identity = self.get_default_forest_keypair()?;
        optional_default_forest_identity.map_or(Ok(None), |default_forest_value| {
            Ok(Some(WildlandIdentity::Forest(0, default_forest_value)))
        })
    }

    pub fn save_forest_uuid(&self, forest: &dyn ForestManifest) -> LssResult<bool> {
        tracing::trace!("Saving forest uuid");
        self.serialize_and_save(forest.owner().encode(), &forest.uuid())
    }

    pub fn get_forest_uuid_by_identity(
        &self,
        forest_identity: &WildlandIdentity,
    ) -> LssResult<Option<Uuid>> {
        self.get_parsed(Identity::from(forest_identity.get_public_key()).encode())
    }

    pub fn get_this_device_identity(&self) -> LssResult<Option<WildlandIdentity>> {
        tracing::trace!("Getting this device identity.");

        let optional_this_device_identity = self.get_this_device_keypair()?;
        optional_this_device_identity.map_or(Ok(None), |this_device_identity| {
            let device_name = self.get_this_device_name()?.ok_or_else(|| {
                LssError::Error("Could not retrieve device name from LSS".to_owned())
            })?;
            Ok(Some(WildlandIdentity::Device(
                device_name,
                this_device_identity,
            )))
        })
    }

    fn get_this_device_name(&self) -> LssResult<Option<String>> {
        self.get_parsed(THIS_DEVICE_NAME_KEY)
    }

    fn get_this_device_keypair(&self) -> LssResult<Option<SigningKeypair>> {
        self.get_parsed(THIS_DEVICE_KEYPAIR_KEY)
    }

    fn get_default_forest_keypair(&self) -> LssResult<Option<SigningKeypair>> {
        self.get_parsed(DEFAULT_FOREST_KEY)
    }

    /// serializes an `obj` and saves it in LSS
    /// this is the only method which should use `serde_json::to_vec` function so it could be easily replaced
    fn serialize_and_save(
        &self,
        key: impl Display + Debug,
        obj: &impl Serialize,
    ) -> LssResult<bool> {
        self.lss
            .insert(
                key.to_string(),
                serde_json::to_string(obj)
                    .map_err(|e| LssError::Error(format!("Could not serialize object: {e}")))?,
            )
            .map(|bytes| bytes.is_some())
    }

    /// retrieves bytes from LSS, deserializes them as json and parses as a type specified with template parameter
    /// this is the only method which should use `serde_json::from_slice` function so it could be easily replaced
    fn get_parsed<T: DeserializeOwned>(&self, key: impl Display + Debug) -> LssResult<Option<T>> {
        self.lss.get(key.to_string()).and_then(|optional_bytes| {
            optional_bytes.map_or(Ok(None), |input| {
                serde_json::from_str(&input)
                    .map_err(|e| LssError::Error(format!("Could not parse LSS entry: {e}")))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use rstest::{fixture, rstest};
    use uuid::Uuid;
    use wildland_crypto::identity::SigningKeypair;

    use crate::catlib_service::entities::{ForestManifest as IForest, Identity};
    use crate::lss::service::{THIS_DEVICE_KEYPAIR_KEY, THIS_DEVICE_NAME_KEY};
    use crate::test_utils::MockForest;
    use crate::{LocalSecureStorage, LssResult, LssService, WildlandIdentity, DEFAULT_FOREST_KEY};

    #[fixture]
    fn lss_stub() -> &'static dyn LocalSecureStorage {
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

    #[rstest]
    fn test_save_forest_identity(lss_stub: &'static dyn LocalSecureStorage) {
        let service = LssService::new(lss_stub);

        let keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();
        let forest_identity = WildlandIdentity::Forest(5, SigningKeypair::from(&keypair));
        service.save_identity(&forest_identity).unwrap();

        let expected_key = "wildland.forest.5".to_string();

        let deserialized_keypair: SigningKeypair =
            serde_json::from_str(&lss_stub.get(expected_key).unwrap().unwrap()).unwrap();
        assert_eq!(deserialized_keypair, keypair);
    }

    #[rstest]
    fn test_save_device_identity(lss_stub: &'static dyn LocalSecureStorage) {
        let service = LssService::new(lss_stub);

        let device_name = "some device".to_owned();
        let keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();
        let device_identity =
            WildlandIdentity::Device(device_name.clone(), SigningKeypair::from(&keypair));
        service.save_identity(&device_identity).unwrap();

        let deserialized_keypair: SigningKeypair = serde_json::from_str(
            &lss_stub
                .get(THIS_DEVICE_KEYPAIR_KEY.to_string())
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(deserialized_keypair, keypair);

        let deserialized_name: String = serde_json::from_str(
            &lss_stub
                .get(THIS_DEVICE_NAME_KEY.to_owned())
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(deserialized_name, device_name);
    }

    #[rstest]
    fn get_default_forest_should_return_none(lss_stub: &'static dyn LocalSecureStorage) {
        let service = LssService::new(lss_stub);

        let default_forest = service.get_default_forest_identity().unwrap();

        assert!(default_forest.is_none())
    }

    #[rstest]
    fn get_default_forest_should_return_identity(lss_stub: &'static dyn LocalSecureStorage) {
        let service = LssService::new(lss_stub);

        let keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();
        lss_stub
            .insert(
                DEFAULT_FOREST_KEY.to_owned(),
                serde_json::to_string(&keypair).unwrap(),
            )
            .unwrap();

        let default_forest = service.get_default_forest_identity().unwrap();

        let expecte_forest_identity = WildlandIdentity::Forest(0, SigningKeypair::from(&keypair));
        assert_eq!(default_forest.unwrap(), expecte_forest_identity)
    }

    #[rstest]
    fn test_save_forest_uuid(lss_stub: &'static dyn LocalSecureStorage) {
        let service = LssService::new(lss_stub);

        let forest_identity = Identity([1; 32]);
        let uuid = Uuid::new_v4();
        let mut forest = MockForest::new();
        forest.expect_owner().returning({
            let fi = forest_identity.clone();
            move || fi.clone()
        });
        forest.expect_uuid().returning(move || uuid);

        service.save_forest_uuid(&forest).unwrap();

        let retrieved_uuid: Uuid =
            serde_json::from_str(&lss_stub.get(forest_identity.encode()).unwrap().unwrap())
                .unwrap();

        assert_eq!(retrieved_uuid, forest.uuid());
    }

    #[rstest]
    fn test_get_forest_uuid_by_identity(lss_stub: &'static dyn LocalSecureStorage) {
        let service = LssService::new(lss_stub);

        let forest_uuid = Uuid::new_v4();
        let forest_pubkey = [1; 32];
        let forest_identity = Identity(forest_pubkey);
        lss_stub
            .insert(
                forest_identity.encode(),
                serde_json::to_string(&forest_uuid).unwrap(),
            )
            .unwrap();

        let retrieved_uuid = service
            .get_forest_uuid_by_identity(&WildlandIdentity::Forest(
                5,
                SigningKeypair::try_from_bytes_slices(forest_pubkey, [2; 32]).unwrap(),
            ))
            .unwrap()
            .unwrap();

        assert_eq!(retrieved_uuid, forest_uuid);
    }

    #[rstest]
    fn test_get_this_device_identity_should_return_none(lss_stub: &'static dyn LocalSecureStorage) {
        let service = LssService::new(lss_stub);

        let device_identity = service.get_this_device_identity().unwrap();

        assert!(device_identity.is_none())
    }

    #[rstest]
    fn test_get_this_device_identity_should_return_identity(
        lss_stub: &'static dyn LocalSecureStorage,
    ) {
        let service = LssService::new(lss_stub);

        let device_name = "some device".to_owned();
        let keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();

        lss_stub
            .insert(
                THIS_DEVICE_NAME_KEY.to_owned(),
                serde_json::to_string(&device_name).unwrap(),
            )
            .unwrap();
        lss_stub
            .insert(
                THIS_DEVICE_KEYPAIR_KEY.to_owned(),
                serde_json::to_string(&keypair).unwrap(),
            )
            .unwrap();

        let device_identity = service.get_this_device_identity().unwrap().unwrap();
        let expected_device_identity =
            WildlandIdentity::Device(device_name, SigningKeypair::from(&keypair));
        assert_eq!(device_identity, expected_device_identity);
    }
}
