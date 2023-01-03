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

use crate::{api::container::*, errors::storage::GetStorageTemplateError};
use derivative::Derivative;
use uuid::Uuid;
use wildland_corex::{
    catlib_service::{entities::ForestManifest, error::CatlibError, CatLibService},
    StorageTemplate,
};

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

/// Structure representing a User.
///
/// It gives access to:
/// - user's forest and containers,
/// - Foundation Storage API which includes the following methods:
///     - [`Self::request_free_tier_storage()`]
///     - [`Self::verify_email()`]
#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct CargoUser {
    this_device: String,
    all_devices: Vec<String>,

    forest: Box<dyn ForestManifest>,

    #[derivative(Debug = "ignore")]
    catlib_service: CatLibService,
    #[derivative(Debug = "ignore")]
    user_context: UserContext,
    #[derivative(Debug = "ignore")]
    fsa_api: FoundationStorageApi,
}

impl CargoUser {
    pub fn new(
        this_device: String,
        all_devices: Vec<String>,
        forest: Box<dyn ForestManifest>,
        catlib_service: CatLibService,
        fsa_config: &FoundationStorageApiConfig,
    ) -> Self {
        Self {
            this_device,
            all_devices,
            forest,
            catlib_service,
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

    /// Returns vector of handles to all containers (mounted or not) found in the user's forest.
    #[tracing::instrument(level = "debug", skip_all)]
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
    #[tracing::instrument(level = "debug", skip_all)]
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
    #[tracing::instrument(level = "debug", skip_all)]
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
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_storage_templates(&self) -> Result<Vec<StorageTemplate>, GetStorageTemplateError> {
        self.catlib_service
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
    #[tracing::instrument(level = "debug", skip_all)]
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
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn verify_email(
        &mut self,
        process_handle: &FreeTierProcessHandle,
        token: String,
    ) -> Result<StorageTemplate, FsaError> {
        let storage_template = process_handle.verify_email(token)?;
        self.catlib_service
            .save_storage_template(&storage_template)?;
        self.catlib_service
            .mark_free_storage_granted(self.forest.as_mut())?;
        Ok(storage_template)
    }

    pub fn is_free_storage_granted(&mut self) -> Result<bool, CatlibError> {
        self.catlib_service
            .is_free_storage_granted(self.forest.as_mut())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mockito::Matcher;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use serde_json::json;
    use uuid::Uuid;
    use wildland_corex::{
        catlib_service::{entities::ForestManifest, CatLibService, DeviceMetadata, ForestMetaData},
        SigningKeypair, WildlandIdentity,
    };

    use crate::{
        api::{config::FoundationStorageApiConfig, utils::test::catlib_service},
        templates::foundation_storage::FoundationStorageTemplate,
    };

    use super::CargoUser;

    #[fixture]
    fn setup(catlib_service: CatLibService) -> (CargoUser, CatLibService, Box<dyn ForestManifest>) {
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

        let forest_copy = catlib_service.get_forest(&forest.uuid()).unwrap();

        let cargo_user = CargoUser::new(
            this_dev_name.clone(),
            vec![this_dev_name],
            forest,
            catlib_service.clone(),
            &FoundationStorageApiConfig {
                evs_url: mockito::server_url(),
                sc_url: "".to_string(),
            },
        );

        (cargo_user, catlib_service, forest_copy)
    }

    #[rstest]
    fn test_requesting_free_tier_storage(
        setup: (CargoUser, CatLibService, Box<dyn ForestManifest>),
    ) {
        // given setup
        let (mut cargo_user, catlib_service, mut forest) = setup;

        // when storage request is sent
        let session_uuid = "00000000-0000-0000-0000-000000000001";
        let session_id_response = format!(
            r#"{{
                "session_id": "{session_uuid}"
            }}"#
        );
        let storage_req_mock_1 = mockito::mock("PUT", "/get_storage")
            .with_status(202)
            .with_body(session_id_response)
            .create();

        let process_handle = cargo_user
            .request_free_tier_storage("test@wildland.io".to_string())
            .unwrap();

        // and verification is performed
        let verify_token_mock = mockito::mock("PUT", "/confirm_token")
            .match_body(Matcher::JsonString(format!(
                r#"{{
                    "email": "test@wildland.io",
                    "verification_token": "123456",
                    "session_id": "{session_uuid}"
                }}"#
            )))
            .with_status(200)
            .create();

        let response_json_str = r#"{"id": "00000000-0000-0000-0000-000000000001", "credentialID": "cred_id", "credentialSecret": "cred_secret"}"#;
        let response_base64 = base64::encode(response_json_str);
        let credentials_response = format!("{{ \"credentials\": \"{response_base64}\" }}");
        let storage_req_mock_2 = mockito::mock("PUT", "/get_storage")
            .with_status(200)
            .with_body(credentials_response)
            .create();

        cargo_user
            .verify_email(&process_handle, "123456".to_string())
            .unwrap();

        // then all http requests expectations are met
        storage_req_mock_1.assert();
        verify_token_mock.assert();
        storage_req_mock_2.assert();

        // and storage template is saved in CatLib
        let template_data = serde_json::from_str::<serde_json::Value>(
            &catlib_service.get_storage_templates_data().unwrap()[0],
        )
        .unwrap();
        let expected_template_json = json!(
            {
                "uuid": template_data["uuid"],
                "backend_type":"FoundationStorage",
                "name": null,
                "template":
                {
                    "bucket_uuid": "00000000-0000-0000-0000-000000000001",
                    "credential_id": "cred_id",
                    "credential_secret": "cred_secret",
                    "sc_url": "",
                    "container_prefix": "{{ OWNER }}/{{ CONTAINER_NAME }}"
                }
            }
        );
        assert_eq!(template_data, expected_template_json);

        // and storage granted flag is set in catlib
        assert!(catlib_service
            .is_free_storage_granted(forest.as_mut())
            .unwrap())
    }

    #[rstest]
    fn test_creating_container(setup: (CargoUser, CatLibService, Box<dyn ForestManifest>)) {
        // given setup
        let (cargo_user, catlib_service, forest) = setup;

        // when container is created
        let container_uuid_str = "00000000-0000-0000-0000-000000000001";
        let storage_template = FoundationStorageTemplate::new(
            Uuid::from_str(container_uuid_str).unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "some url".to_owned(),
        )
        .try_into()
        .unwrap();
        let container_name = "new container".to_string();
        cargo_user
            .create_container(container_name.clone(), &storage_template)
            .unwrap();

        // then it is stored in catlib
        let retrieved_forest = catlib_service.get_forest(&forest.uuid()).unwrap();
        let mut containers = retrieved_forest.containers().unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].name().unwrap(), container_name);
        assert_eq!((*containers[0].forest().unwrap()).uuid(), (*forest).uuid());
    }

    #[rstest]
    fn test_getting_created_container(setup: (CargoUser, CatLibService, Box<dyn ForestManifest>)) {
        // given setup
        let (cargo_user, _catlib_service, _forest) = setup;

        // when container is created
        let container_uuid_str = "00000000-0000-0000-0000-000000000001";
        let storage_template = FoundationStorageTemplate::new(
            Uuid::from_str(container_uuid_str).unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "some url".to_owned(),
        )
        .try_into()
        .unwrap();
        let container_name = "new container".to_string();
        cargo_user
            .create_container(container_name.clone(), &storage_template)
            .unwrap();

        // then it can be retrieved via CargoUser api
        let containers = cargo_user.get_containers().unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!(
            containers[0].lock().unwrap().get_name().unwrap(),
            container_name
        );
    }

    #[rstest]
    fn test_delete_created_container(setup: (CargoUser, CatLibService, Box<dyn ForestManifest>)) {
        // given setup
        let (cargo_user, _catlib_service, _forest) = setup;

        // when a container is created
        let container_uuid_str = "00000000-0000-0000-0000-000000000001";
        let storage_template = FoundationStorageTemplate::new(
            Uuid::from_str(container_uuid_str).unwrap(),
            "cred_id".to_owned(),
            "cred_secret".to_owned(),
            "some url".to_owned(),
        )
        .try_into()
        .unwrap();
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
