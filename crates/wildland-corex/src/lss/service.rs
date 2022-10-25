//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt::{Debug, Display};

use super::{api::LocalSecureStorage, result::LssResult};
use crate::{ForestRetrievalError, LssError, WildlandIdentity, DEFAULT_FOREST_KEY};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;
use wildland_catlib::{Forest, IForest, Identity};
use wildland_crypto::identity::SigningKeypair;

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

    #[tracing::instrument(level = "debug", skip(self, wildland_identity))]
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

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_default_forest_identity(
        &self,
    ) -> Result<Option<WildlandIdentity>, ForestRetrievalError> {
        tracing::trace!("Getting default forest identity.");

        let optional_default_forest_identity = self.get_default_forest_keypair()?;
        optional_default_forest_identity.map_or(Ok(None), |default_forest_value| {
            Ok(Some(WildlandIdentity::Forest(0, default_forest_value)))
        })
    }

    #[tracing::instrument(level = "debug", skip(self, forest))]
    pub fn save_forest_uuid(&self, forest: &Forest) -> LssResult<bool> {
        tracing::trace!("Saving forest uuid");
        self.serialize_and_save(forest.owner().encode(), &forest.uuid())
    }

    #[tracing::instrument(level = "debug", skip(self, forest_identity))]
    pub fn get_forest_uuid_by_identity(
        &self,
        forest_identity: &WildlandIdentity,
    ) -> LssResult<Option<Uuid>> {
        self.get_parsed(Identity::from(forest_identity.get_public_key()).encode())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_this_device_identity(&self) -> LssResult<Option<WildlandIdentity>> {
        tracing::trace!("Getting this device identity.");

        let optional_this_device_identity = self.get_this_device_keypair()?;
        optional_this_device_identity.map_or(Ok(None), |this_device_identity| {
            let device_name = self
                .get_this_device_name()?
                .ok_or_else(|| LssError("Could not retrieve device name from LSS".to_owned()))?;
            Ok(Some(WildlandIdentity::Device(
                device_name,
                this_device_identity,
            )))
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_this_device_name(&self) -> LssResult<Option<String>> {
        self.get_parsed(THIS_DEVICE_NAME_KEY)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_this_device_keypair(&self) -> LssResult<Option<SigningKeypair>> {
        self.get_parsed(THIS_DEVICE_KEYPAIR_KEY)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_default_forest_keypair(&self) -> LssResult<Option<SigningKeypair>> {
        self.get_parsed(DEFAULT_FOREST_KEY)
    }

    #[tracing::instrument(level = "debug", skip(self, obj))]
    fn serialize_and_save(
        &self,
        key: impl Display + Debug,
        obj: &impl Serialize,
    ) -> LssResult<bool> {
        self.lss
            .insert(
                key.to_string(),
                serde_json::to_vec(obj)
                    .map_err(|e| LssError(format!("Could not serialize object: {e}")))?,
            )
            .map(|bytes| bytes.is_some())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_parsed<'a, T: DeserializeOwned>(
        &self,
        key: impl Display + Debug,
    ) -> LssResult<Option<T>> {
        self.lss.get(key.to_string()).and_then(|optional_bytes| {
            optional_bytes.map_or(Ok(None), |bytes| {
                serde_json::from_slice(bytes.as_slice())
                    .map_err(|e| LssError(format!("Could not parse LSS entry: {e}")))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
    };

    use uuid::Uuid;
    use wildland_catlib::{CatLib, IForest, Identity};
    use wildland_crypto::identity::SigningKeypair;

    use crate::{
        lss::service::{THIS_DEVICE_KEYPAIR_KEY, THIS_DEVICE_NAME_KEY},
        LocalSecureStorage, LssResult, LssService, WildlandIdentity, DEFAULT_FOREST_KEY,
    };

    #[derive(Default)]
    struct LssStub {
        storage: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl LocalSecureStorage for LssStub {
        fn insert(&self, key: String, value: Vec<u8>) -> LssResult<Option<Vec<u8>>> {
            Ok(self.storage.borrow_mut().insert(key, value))
        }

        fn get(&self, key: String) -> LssResult<Option<Vec<u8>>> {
            Ok(self.storage.try_borrow().unwrap().get(&key).cloned())
        }

        fn contains_key(&self, key: String) -> LssResult<bool> {
            Ok(self.storage.borrow().contains_key(&key))
        }

        fn keys(&self) -> LssResult<Vec<String>> {
            Ok(self.storage.borrow().keys().cloned().collect())
        }

        fn remove(&self, key: String) -> LssResult<Option<Vec<u8>>> {
            Ok(self.storage.borrow_mut().remove(&key))
        }

        fn len(&self) -> LssResult<usize> {
            Ok(self.storage.borrow().len())
        }

        fn is_empty(&self) -> LssResult<bool> {
            Ok(self.storage.borrow().is_empty())
        }
    }

    #[test]
    fn test_save_forest_identity() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let service = LssService::new(lss_ref);

        let keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();
        let forest_identity = WildlandIdentity::Forest(5, SigningKeypair::from(&keypair));
        service.save_identity(&forest_identity).unwrap();

        let expected_key = "wildland.forest.5".to_string();

        let deserialized_keypair: SigningKeypair =
            serde_json::from_slice(&lss.get(expected_key).unwrap().unwrap()).unwrap();
        assert_eq!(deserialized_keypair, keypair);
    }

    #[test]
    fn test_save_device_identity() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let service = LssService::new(lss_ref);

        let device_name = "some device".to_owned();
        let keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();
        let device_identity =
            WildlandIdentity::Device(device_name.clone(), SigningKeypair::from(&keypair));
        service.save_identity(&device_identity).unwrap();

        let deserialized_keypair: SigningKeypair = serde_json::from_slice(
            &lss.get(THIS_DEVICE_KEYPAIR_KEY.to_string())
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(deserialized_keypair, keypair);

        let deserialized_name: String =
            serde_json::from_slice(&lss.get(THIS_DEVICE_NAME_KEY.to_owned()).unwrap().unwrap())
                .unwrap();
        assert_eq!(deserialized_name, device_name);
    }

    #[test]
    fn get_default_forest_should_return_none() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let service = LssService::new(lss_ref);

        let default_forest = service.get_default_forest_identity().unwrap();

        assert!(default_forest.is_none())
    }

    #[test]
    fn get_default_forest_should_return_identity() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let service = LssService::new(lss_ref);

        let keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();
        lss.insert(
            DEFAULT_FOREST_KEY.to_owned(),
            serde_json::to_vec(&keypair).unwrap(),
        )
        .unwrap();

        let default_forest = service.get_default_forest_identity().unwrap();

        let expecte_forest_identity = WildlandIdentity::Forest(0, SigningKeypair::from(&keypair));
        assert_eq!(default_forest.unwrap(), expecte_forest_identity)
    }

    #[test]
    fn test_save_forest_uuid() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let service = LssService::new(lss_ref);

        let tmp = tempfile::tempdir().unwrap().path().into();
        let catlib = CatLib::new(tmp);

        let forest_identity = Identity([1; 32]);
        let forest = catlib
            .create_forest(forest_identity.clone(), HashSet::new(), vec![])
            .unwrap();

        service.save_forest_uuid(&forest).unwrap();

        let retrieved_uuid: Uuid =
            serde_json::from_slice(&lss.get(forest_identity.encode()).unwrap().unwrap()).unwrap();

        assert_eq!(retrieved_uuid, forest.uuid());
    }

    #[test]
    fn test_get_forest_uuid_by_identity() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let service = LssService::new(lss_ref);

        let forest_uuid = Uuid::new_v4();
        let forest_pubkey = [1; 32];
        let forest_identity = Identity(forest_pubkey);
        lss.insert(
            forest_identity.encode(),
            serde_json::to_vec(&forest_uuid).unwrap(),
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

    #[test]
    fn test_get_this_device_identity_should_return_none() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let service = LssService::new(lss_ref);

        let device_identity = service.get_this_device_identity().unwrap();

        assert!(device_identity.is_none())
    }

    #[test]
    fn test_get_this_device_identity_should_return_identity() {
        let lss = LssStub::default(); // LSS must live through the whole test
        let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
        let service = LssService::new(lss_ref);

        let device_name = "some device".to_owned();
        let keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();

        lss.insert(
            THIS_DEVICE_NAME_KEY.to_owned(),
            serde_json::to_vec(&device_name).unwrap(),
        )
        .unwrap();
        lss.insert(
            THIS_DEVICE_KEYPAIR_KEY.to_owned(),
            serde_json::to_vec(&keypair).unwrap(),
        )
        .unwrap();

        let device_identity = service.get_this_device_identity().unwrap().unwrap();
        let expected_device_identity =
            WildlandIdentity::Device(device_name, SigningKeypair::from(&keypair));
        assert_eq!(device_identity, expected_device_identity);
    }
}
