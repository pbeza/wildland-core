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
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    api::{container::*, storage_template::*},
    errors::{storage::GetStorageTemplateError, user::*},
};
use derivative::Derivative;
use uuid::Uuid;
use wildland_corex::catlib_service::{entities::Forest, error::CatlibError, CatLibService};
use wildland_corex::LssService;

use super::{
    config::FoundationStorageApiConfig,
    foundation_storage::{FoundationStorageApi, FreeTierProcessHandle, FsaError},
};

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
use wasm_bindgen::prelude::*;
/// Structure representing a User.
///
/// It gives access to user's forest and containers.
#[derive(Clone, Derivative)]
#[derivative(Debug)]
#[wasm_bindgen]
pub struct CargoUser {
    this_device: String,
    all_devices: Vec<String>,

    forest: Rc<dyn Forest>,

    #[derivative(Debug = "ignore")]
    catlib_service: CatLibService,
    #[derivative(Debug = "ignore")]
    lss_service: LssService,
    #[derivative(Debug = "ignore")]
    user_context: UserContext,
    #[derivative(Debug = "ignore")]
    fsa_api: FoundationStorageApi,
}

impl CargoUser {
    pub fn new(
        this_device: String,
        all_devices: Vec<String>,
        forest: Box<dyn Forest>,
        catlib_service: CatLibService,
        lss_service: LssService,
        fsa_config: &FoundationStorageApiConfig,
    ) -> Self {
        Self {
            this_device,
            all_devices,
            forest: forest.into(),
            catlib_service,
            lss_service,
            user_context: UserContext::new(),
            fsa_api: FoundationStorageApi::new(fsa_config),
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
    pub fn mount_forest(&self) -> Result<(), ForestMountError> {
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
            .create_container(name, self.forest.as_ref(), template)
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
                serde_json::from_str(&st_data)
                    .map_err(|e| GetStorageTemplateError::DeserializationError(e.to_string()))
            })
            .collect()
    }

    /// Starts process of granting Free Tier Foundation Storage.
    ///
    /// `CargoUser` encapsulates `FoundationStorageApi` functionalities in order to avoid requesting
    /// Free Foundation Tier outside of the user context.
    ///
    /// Returns `FreeTierProcessHandle` structure which can be used to verify an email address and
    /// finalize the process.
    pub fn request_free_tier_storage(
        &self,
        email: String,
    ) -> Result<FreeTierProcessHandle, FsaError> {
        self.fsa_api.request_free_tier_storage(email)
    }

    /// Finishes process of granting Free Tier Foundation Storage.
    ///
    /// After successful server verification it saves Storage Template in LSS
    /// and saves information that storage has been granted in forest's metadata in CatLib.
    ///
    /// Returns the same storage template which is saved in LSS.
    pub fn verify_email(
        &mut self,
        process_handle: &FreeTierProcessHandle,
        token: String,
    ) -> Result<StorageTemplate, FsaError> {
        let storage_template = process_handle.verify_email(token)?;
        self.lss_service.save_storage_template(&storage_template)?;
        self.catlib_service
            .mark_free_storage_granted(
                Rc::get_mut(&mut self.forest)
                .ok_or_else(||
                    FsaError::Generic("Could mutate user's state - probably there is more than one user's handle which is considered as not safe".to_string())
                )?
            )?;
        Ok(storage_template)
    }

    pub fn is_free_storage_granted(&self) -> Result<bool, CatlibError> {
        self.catlib_service
            .is_free_storage_granted(self.forest.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mockito::Matcher;
    use rstest::*;
    use uuid::Uuid;
    use wildland_corex::catlib_service::entities::Forest;
    use wildland_corex::{
        CatLibService, DeviceMetadata, ForestMetaData, LocalSecureStorage, LssService,
        SigningKeypair, WildlandIdentity,
    };

    use crate::api::config::FoundationStorageApiConfig;
    use crate::api::utils::test::{catlib_service, lss_stub};
    use crate::api::{
        foundation_storage::FoundationStorageTemplate, storage_template::StorageTemplate,
    };

    use super::CargoUser;

    #[fixture]
    fn setup(
        catlib_service: CatLibService,
        lss_stub: &'static dyn LocalSecureStorage,
    ) -> (CargoUser, CatLibService, Box<dyn Forest>) {
        let lss_service = LssService::new(lss_stub);

        let this_dev_name = "My device".to_string();

        let forest_keypair = SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap();
        let forest_identity = WildlandIdentity::Forest(5, SigningKeypair::from(&forest_keypair));
        let device_keypair = SigningKeypair::try_from_bytes_slices([3; 32], [4; 32]).unwrap();
        let device_identity =
            WildlandIdentity::Device(this_dev_name.clone(), SigningKeypair::from(&device_keypair));
        let forest = catlib_service
            .add_forest(
                &forest_identity,
                &device_identity,
                ForestMetaData::new(vec![DeviceMetadata {
                    name: this_dev_name.clone(),
                    pubkey: device_keypair.public(),
                }]),
            )
            .unwrap();

        let forest_copy = catlib_service.get_forest(&(*forest).as_ref().uuid).unwrap();

        let cargo_user = CargoUser::new(
            this_dev_name.clone(),
            vec![this_dev_name],
            forest,
            catlib_service.clone(),
            lss_service,
            &FoundationStorageApiConfig {
                evs_url: mockito::server_url(),
                sc_url: "".to_string(),
            },
        );

        (cargo_user, catlib_service, forest_copy)
    }

    #[rstest]
    fn test_http_requests_to_evs(setup: (CargoUser, CatLibService, Box<dyn Forest>)) {
        // given setup
        let (mut cargo_user, _catlib_service, _forest) = setup;

        // when storage request is sent
        let storage_req_mock_1 = mockito::mock("PUT", "/get_storage")
            .with_status(202)
            .create();

        let process_handle = cargo_user
            .request_free_tier_storage("test@wildland.io".to_string())
            .unwrap();

        // and verification is performed
        let verify_token_mock = mockito::mock("PUT", "/confirm_token")
            .match_body(Matcher::JsonString(
                "{\"email\":\"test@wildland.io\",\"verification_token\":\"123456\"}".to_string(),
            ))
            .with_status(200)
            .create();

        let response_json_str = r#"{"id": "00000000-0000-0000-0000-000000000000", "credentialID": "cred_id", "credentialSecret": "cred_secret"}"#;
        let response_base64 = base64::encode(response_json_str);
        let full_response = format!("{{ \"encrypted_credentials\": \"{response_base64}\" }}");
        let storage_req_mock_2 = mockito::mock("PUT", "/get_storage")
            .with_status(200)
            .with_body(full_response)
            .create();

        cargo_user
            .verify_email(&process_handle, "123456".to_string())
            .unwrap();

        // then all http requests expectations are met

        storage_req_mock_1.assert();
        verify_token_mock.assert();
        storage_req_mock_2.assert();
    }

    #[rstest]
    fn test_creating_container(setup: (CargoUser, CatLibService, Box<dyn Forest>)) {
        // given setup
        let (cargo_user, catlib_service, forest) = setup;

        // when container is created
        let container_uuid_str = "00000000-0000-0000-0000-000000000001";
        let storage_template =
            StorageTemplate::FoundationStorageTemplate(FoundationStorageTemplate {
                uuid: Uuid::from_str(container_uuid_str).unwrap(),
                credential_id: "cred_id".to_owned(),
                credential_secret: "cred_secret".to_owned(),
                sc_url: "some url".to_owned(),
            });
        let container_name = "new container".to_string();
        cargo_user
            .create_container(container_name.clone(), &storage_template)
            .unwrap();

        // then it is stored in catlib
        let retrieved_forest = catlib_service.get_forest(&(*forest).as_ref().uuid).unwrap();
        let containers = retrieved_forest.containers().unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!((*containers[0]).as_ref().name, container_name);
        assert_eq!(
            (*containers[0].forest().unwrap()).as_ref().uuid,
            (*forest).as_ref().uuid
        );
    }

    #[rstest]
    fn test_getting_created_container(setup: (CargoUser, CatLibService, Box<dyn Forest>)) {
        // given setup
        let (cargo_user, _catlib_service, _forest) = setup;

        // when container is created
        let container_uuid_str = "00000000-0000-0000-0000-000000000001";
        let storage_template =
            StorageTemplate::FoundationStorageTemplate(FoundationStorageTemplate {
                uuid: Uuid::from_str(container_uuid_str).unwrap(),
                credential_id: "cred_id".to_owned(),
                credential_secret: "cred_secret".to_owned(),
                sc_url: "some url".to_owned(),
            });
        let container_name = "new container".to_string();
        cargo_user
            .create_container(container_name.clone(), &storage_template)
            .unwrap();

        // then it can be retrieved via CargoUser api
        let containers = cargo_user.get_containers().unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].lock().unwrap().get_name(), container_name);
    }

    #[rstest]
    fn test_delete_created_container(setup: (CargoUser, CatLibService, Box<dyn Forest>)) {
        // given setup
        let (cargo_user, _catlib_service, _forest) = setup;

        // when a container is created
        let container_uuid_str = "00000000-0000-0000-0000-000000000001";
        let storage_template =
            StorageTemplate::FoundationStorageTemplate(FoundationStorageTemplate {
                uuid: Uuid::from_str(container_uuid_str).unwrap(),
                credential_id: "cred_id".to_owned(),
                credential_secret: "cred_secret".to_owned(),
                sc_url: "some url".to_owned(),
            });
        let container_name = "new container".to_string();
        let container = cargo_user
            .create_container(container_name, &storage_template)
            .unwrap();

        // and the container is deleted
        cargo_user.delete_container(&container).unwrap();

        // then it cannot be retrieved via CargoUser api
        let containers = cargo_user.get_containers().unwrap();
        assert_eq!(containers.len(), 0);

        // and the container handle received during creation is marked as deleted
        assert!(container.lock().unwrap().is_deleted());
    }
}
