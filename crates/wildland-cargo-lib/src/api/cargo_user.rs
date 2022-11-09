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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    api::{container::*, storage_template::*},
    errors::{single_variant::*, storage::GetStorageTemplateError, user::*},
};
use derivative::Derivative;
use uuid::Uuid;
use wildland_corex::{CatLibService, CatlibError, Forest, IForest, LssService};

pub type SharedContainer = Arc<Mutex<Container>>;
#[derive(Debug, Clone)]
struct UserContext {
    loaded_containers: Arc<Mutex<HashMap<Uuid, SharedContainer>>>,
}

impl UserContext {
    fn new() -> Self {
        Self {
            loaded_containers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add_container(&self, uuid: Uuid, container: SharedContainer) {
        self.loaded_containers
            .lock()
            .expect("Could not lock loaded containers in user context")
            .insert(uuid, container);
    }

    fn get_loaded_container(&self, uuid: Uuid) -> Option<SharedContainer> {
        self.loaded_containers
            .lock()
            .expect("Could not lock loaded containers in user context")
            .get(&uuid)
            .cloned()
    }
}

/// Structure representing a User.
///
/// It gives access to user's forest and containers.
#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct CargoUser {
    this_device: String,
    all_devices: Vec<String>,

    forest: Forest,

    #[derivative(Debug = "ignore")]
    catlib_service: CatLibService,
    #[derivative(Debug = "ignore")]
    lss_service: LssService,
    #[derivative(Debug = "ignore")]
    user_context: UserContext,
}

impl CargoUser {
    pub fn new(
        this_device: String,
        all_devices: Vec<String>,
        forest: Forest,
        catlib_service: CatLibService,
        lss_service: LssService,
    ) -> Self {
        Self {
            this_device,
            all_devices,
            forest,
            catlib_service,
            lss_service,
            user_context: UserContext::new(),
        }
    }

    /// Returns string representation of a [`CargoUser`]
    pub fn stringify(&self) -> String {
        let CargoUser {
            this_device,
            all_devices,
            ..
        } = &self;
        let all_devices_str = all_devices
            .iter()
            .map(|d| format!("    {d}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "
This device: {this_device}
All devices:
{all_devices_str}
"
        )
    }

    /// TODO
    pub fn mount_forest(&self) -> SingleErrVariantResult<(), ForestMountError> {
        todo!()
    }

    /// Returns vector of handles to all containers (mounted or not) found in the user's forest.
    pub fn get_containers(&self) -> Result<Vec<Arc<Mutex<Container>>>, CatlibError> {
        match self.forest.containers() {
            Ok(inner_containers) => {
                Ok(inner_containers
                    .into_iter()
                    .map(|c| {
                        let container = Container::from(c);
                        let uuid = container.uuid();
                        if let Some(cached_container) = self.user_context.get_loaded_container(uuid)
                        {
                            // update container in user's context with the content fetched from CatLib
                            let mut locked_cached_container = cached_container
                                .lock()
                                .expect("Could not lock loaded container in user context");
                            *locked_cached_container = container;
                            cached_container.clone()
                        } else {
                            // create container in context if it was not already there
                            let shared = Arc::new(Mutex::new(container));
                            self.user_context.add_container(uuid, shared.clone());
                            shared
                        }
                    })
                    .collect::<Vec<SharedContainer>>())
            }
            Err(err) => match err {
                CatlibError::NoRecordsFound => Ok(vec![]),
                _ => Err(err),
            },
        }
    }

    /// Creates a new container within user's forest and return its handle
    pub fn create_container(
        &self,
        name: String,
        template: &StorageTemplate,
    ) -> Result<SharedContainer, CatlibError> {
        let container: Container = self
            .catlib_service
            .create_container(name, &self.forest, template)
            .map(|c| c.into())?;
        let container_uuid = container.uuid();
        let shared_container = Arc::new(Mutex::new(container));
        self.user_context
            .add_container(container_uuid, shared_container.clone());
        Ok(shared_container)
    }

    /// Deleting container is exposed via this method on `CargoUser`
    /// because in future it may require some additional changes in user's context
    /// (which `Container` structure has no access to).
    ///
    pub fn delete_container(&self, container: &SharedContainer) -> Result<(), CatlibError> {
        container
            .lock()
            .expect("Could not lock shared container while deleting")
            .delete(&self.catlib_service)
    }

    pub fn this_device(&self) -> &str {
        &self.this_device
    }

    pub fn all_devices(&self) -> &[String] {
        self.all_devices.as_slice()
    }

    /// Returns vector of user's storage templates
    ///
    pub fn get_storage_templates(&self) -> Result<Vec<StorageTemplate>, GetStorageTemplateError> {
        self.lss_service
            .get_storage_templates_data()?
            .into_iter()
            .map(|st_data| {
                serde_json::from_slice(&st_data)
                    .map_err(|e| GetStorageTemplateError::DeserializationError(e.to_string()))
            })
            .collect()
    }
}

// TODO WILX-302 creating, retrieving and deleting container tests are commented because
// CatLib uses for now a single file for all tests which can be run in parallel, hence tests race
// to the database file
// #[cfg(test)]
// mod tests {
//     use std::{cell::RefCell, collections::HashMap, str::FromStr};

//     use rstest::*;
//     use uuid::Uuid;
//     use wildland_corex::{
//         CatLibService, DeviceMetadata, Forest, IContainer, IForest, LocalSecureStorage, LssResult,
//         LssService, SigningKeypair, UserMetaData, WildlandIdentity,
//     };

//     use crate::api::{
//         foundation_storage::FoundationStorageTemplate, storage_template::StorageTemplate,
//     };

//     use super::CargoUser;

//     #[derive(Default)]
//     struct LssStub {
//         storage: RefCell<HashMap<String, Vec<u8>>>,
//     }

//     impl LocalSecureStorage for LssStub {
//         fn insert(&self, key: String, value: Vec<u8>) -> LssResult<Option<Vec<u8>>> {
//             Ok(self.storage.borrow_mut().insert(key, value))
//         }

//         fn get(&self, key: String) -> LssResult<Option<Vec<u8>>> {
//             Ok(self.storage.try_borrow().unwrap().get(&key).cloned())
//         }

//         fn contains_key(&self, key: String) -> LssResult<bool> {
//             Ok(self.storage.borrow().contains_key(&key))
//         }

//         fn keys(&self) -> LssResult<Vec<String>> {
//             Ok(self.storage.borrow().keys().cloned().collect())
//         }

//         fn keys_starting_with(&self, prefix: String) -> LssResult<Vec<String>> {
//             Ok(self
//                 .storage
//                 .borrow()
//                 .keys()
//                 .filter(|key| key.starts_with(&prefix))
//                 .cloned()
//                 .collect())
//         }

//         fn remove(&self, key: String) -> LssResult<Option<Vec<u8>>> {
//             Ok(self.storage.borrow_mut().remove(&key))
//         }

//         fn len(&self) -> LssResult<usize> {
//             Ok(self.storage.borrow().len())
//         }

//         fn is_empty(&self) -> LssResult<bool> {
//             Ok(self.storage.borrow().is_empty())
//         }
//     }

//     #[fixture]
//     fn setup() -> (CargoUser, CatLibService, Forest) {
//         let lss = LssStub::default(); // LSS must live through the whole test
//         let lss_ref: &'static LssStub = unsafe { std::mem::transmute(&lss) };
//         let lss_service = LssService::new(lss_ref);

//         let catlib_service = CatLibService::new();

//         let this_dev_name = "My device".to_string();

//         let forest_keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();
//         let forest_identity = WildlandIdentity::Forest(5, SigningKeypair::from(&forest_keypair));
//         let device_keypair = SigningKeypair::try_from_bytes_slices([3; 32], [4; 32]).unwrap();
//         let device_identity =
//             WildlandIdentity::Device(this_dev_name.clone(), SigningKeypair::from(&device_keypair));
//         let forest = catlib_service
//             .add_forest(
//                 &forest_identity,
//                 &device_identity,
//                 UserMetaData {
//                     devices: vec![DeviceMetadata {
//                         name: this_dev_name.clone(),
//                         pubkey: device_keypair.public(),
//                     }],
//                 },
//             )
//             .unwrap();

//         let cargo_user = CargoUser::new(
//             this_dev_name.clone(),
//             vec![this_dev_name],
//             forest.clone(),
//             catlib_service.clone(),
//             lss_service,
//         );

//         (cargo_user, catlib_service, forest)
//     }

//     #[rstest]
//     fn test_creating_container(setup: (CargoUser, CatLibService, Forest)) {
//         // given setup
//         let (cargo_user, catlib_service, forest) = setup;

//         // when container is created
//         let container_uuid_str = "00000000-0000-0000-0000-000000000001";
//         let storage_template =
//             StorageTemplate::FoundationStorageTemplate(FoundationStorageTemplate {
//                 uuid: Uuid::from_str(&container_uuid_str).unwrap(),
//                 credential_id: "cred_id".to_owned(),
//                 credential_secret: "cred_secret".to_owned(),
//                 sc_url: "some url".to_owned(),
//             });
//         let container_name = "new container".to_string();
//         cargo_user
//             .create_container(container_name.clone(), &storage_template)
//             .unwrap();

//         // then it is stored in catlib
//         let retrieved_forest = catlib_service.get_forest(forest.uuid()).unwrap();
//         let containers = retrieved_forest.containers().unwrap();
//         assert_eq!(containers.len(), 1);
//         assert_eq!(containers[0].name(), container_name);
//         assert_eq!(containers[0].forest().unwrap().uuid(), forest.uuid());
//     }

//     #[rstest]
//     fn test_getting_created_container(setup: (CargoUser, CatLibService, Forest)) {
//         // given setup
//         let (cargo_user, _catlib_service, _forest) = setup;

//         // when container is created
//         let container_uuid_str = "00000000-0000-0000-0000-000000000001";
//         let storage_template =
//             StorageTemplate::FoundationStorageTemplate(FoundationStorageTemplate {
//                 uuid: Uuid::from_str(&container_uuid_str).unwrap(),
//                 credential_id: "cred_id".to_owned(),
//                 credential_secret: "cred_secret".to_owned(),
//                 sc_url: "some url".to_owned(),
//             });
//         let container_name = "new container".to_string();
//         cargo_user
//             .create_container(container_name.clone(), &storage_template)
//             .unwrap();

//         // then it can be retrieved via CargoUser api
//         let containers = cargo_user.get_containers().unwrap();
//         assert_eq!(containers.len(), 1);
//         assert_eq!(containers[0].lock().unwrap().get_name(), container_name);
//     }

//     #[rstest]
//     fn test_delete_created_container(setup: (CargoUser, CatLibService, Forest)) {
//         // given setup
//         let (cargo_user, _catlib_service, _forest) = setup;

//         // when a container is created
//         let container_uuid_str = "00000000-0000-0000-0000-000000000001";
//         let storage_template =
//             StorageTemplate::FoundationStorageTemplate(FoundationStorageTemplate {
//                 uuid: Uuid::from_str(&container_uuid_str).unwrap(),
//                 credential_id: "cred_id".to_owned(),
//                 credential_secret: "cred_secret".to_owned(),
//                 sc_url: "some url".to_owned(),
//             });
//         let container_name = "new container".to_string();
//         let container = cargo_user
//             .create_container(container_name.clone(), &storage_template)
//             .unwrap();

//         // and the container is deleted
//         cargo_user.delete_container(&container).unwrap();

//         // then it cannot be retrieved via CargoUser api
//         let containers = cargo_user.get_containers().unwrap();
//         assert_eq!(containers.len(), 0);

//         // and the container handle received during creation is marked as deleted
//         assert!(container.lock().unwrap().is_deleted());
//     }
// }
