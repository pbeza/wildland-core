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

use derivative::Derivative;
use wildland_corex::catlib_service::error::CatlibError;
use wildland_corex::catlib_service::CatLibService;
use wildland_corex::{Container, ContainerManager, ContainerManagerError, Forest, StorageTemplate};

use super::config::FoundationStorageApiConfig;
use super::foundation_storage::{FoundationStorageApi, FreeTierProcessHandle, FsaError};
use crate::errors::storage::GetStorageTemplateError;

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
    forest: Forest,

    #[derivative(Debug = "ignore")]
    catlib_service: CatLibService,
    #[derivative(Debug = "ignore")]
    fsa_api: FoundationStorageApi,
    #[derivative(Debug = "ignore")]
    container_manager: ContainerManager,
}

impl CargoUser {
    pub fn new(
        this_device: String,
        all_devices: Vec<String>,
        forest: Forest,
        catlib_service: CatLibService,
        fsa_config: &FoundationStorageApiConfig,
        container_manager: ContainerManager,
    ) -> Self {
        Self {
            this_device,
            all_devices,
            forest,
            catlib_service,
            fsa_api: FoundationStorageApi::new(fsa_config),
            container_manager,
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
    pub fn get_containers(&self) -> Result<Vec<Container>, CatlibError> {
        self.forest.containers()
    }

    /// Creates a new container within user's forest and returns its handle
    ///
    /// ## Example
    ///
    /// ```
    /// # use wildland_cargo_lib::api::CargoConfig;
    /// # use wildland_cargo_lib::api::cargo_lib::create_cargo_lib;
    /// # use wildland_lfs::template::LocalFilesystemStorageTemplate;
    /// # use wildland_corex::StorageTemplate;
    /// # use wildland_cargo_lib::utils::test::lss_stub;
    ///
    /// let tmpdir = tempfile::tempdir().unwrap().into_path();
    ///
    /// let config_str = r#"{
    ///     "log_level": "trace",
    ///     "log_use_ansi": false,
    ///     "log_file_enabled": true,
    ///     "log_file_path": "cargo_lib_log",
    ///     "log_file_rotate_directory": ".",
    ///     "evs_url": "some_url",
    ///     "sc_url": "some_url"
    /// }"#;
    /// let cfg: CargoConfig = serde_json::from_str(config_str).unwrap();
    ///
    /// let lss_stub = lss_stub();
    ///
    /// let cargo_lib = create_cargo_lib(lss_stub, cfg).unwrap();
    /// let cargo_lib = cargo_lib.lock().unwrap();
    /// let user_api = cargo_lib.user_api();
    /// let mnemonic = user_api.generate_mnemonic().unwrap();
    /// let user = user_api
    ///     .create_user_from_mnemonic(&mnemonic, "device_name".to_string())
    ///     .unwrap();
    ///
    /// let template = LocalFilesystemStorageTemplate {
    ///     local_dir: tmpdir.clone(),
    ///     container_prefix: "{{ CONTAINER_NAME }}".to_owned(),
    /// };
    /// let container = user
    ///     .create_container(
    ///         "C1".to_owned(),
    ///         &StorageTemplate::try_new("LocalFilesystem", &template).unwrap(),
    ///         "/some/path/".to_owned(),
    ///     )
    ///     .unwrap();
    /// ```
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_container(
        &self,
        name: String,
        template: &StorageTemplate,
        path: String,
    ) -> Result<Container, CatlibError> {
        self.forest.create_container(name, template, path.into())
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

    /// Save StorageTemplate data in CatLib.
    ///
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save_storage_template(&self, tpl: &StorageTemplate) -> Result<String, CatlibError> {
        self.catlib_service.save_storage_template(tpl)?;

        Ok(tpl.uuid().to_string())
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
            .mark_free_storage_granted(&self.forest.forest_manifest())?;
        Ok(storage_template)
    }

    pub fn is_free_storage_granted(&mut self) -> Result<bool, CatlibError> {
        self.catlib_service
            .is_free_storage_granted(&self.forest.forest_manifest())
    }

    /// Mounts a container in Wildland local device state. After this operation container data
    /// should be accessible via DFS API.
    pub fn mount(&self, container: &Container) -> Result<(), ContainerManagerError> {
        self.container_manager.mount(container)
    }

    /// Unmounts a container from the Wildland local device state. After this operation container
    /// data is not accessible anymore.
    pub fn unmount(&self, container: &Container) -> Result<(), ContainerManagerError> {
        self.container_manager.unmount(container)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use mockito::Matcher;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use serde_json::json;
    use uuid::Uuid;
    use wildland_corex::catlib_service::error::CatlibError;
    use wildland_corex::catlib_service::{CatLibService, DeviceMetadata, ForestMetaData};
    use wildland_corex::{ContainerManager, Forest, SigningKeypair, WildlandIdentity};

    use super::CargoUser;
    use crate::api::config::FoundationStorageApiConfig;
    use crate::templates::foundation_storage::FoundationStorageTemplate;
    use crate::utils::test::catlib_service;

    #[fixture]
    fn setup(catlib_service: CatLibService) -> (CargoUser, CatLibService, Forest) {
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
        let forest = Forest::new(forest);

        let cargo_user = CargoUser::new(
            this_dev_name.clone(),
            vec![this_dev_name],
            forest.clone(),
            catlib_service.clone(),
            &FoundationStorageApiConfig {
                evs_url: mockito::server_url(),
                sc_url: "".to_string(),
            },
            ContainerManager::default(),
        );

        (cargo_user, catlib_service, forest)
    }

    #[rstest]
    fn test_requesting_free_tier_storage(setup: (CargoUser, CatLibService, Forest)) {
        // given setup
        let (mut cargo_user, catlib_service, forest) = setup;

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
        let response_base64 = STANDARD.encode(response_json_str);
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
            .is_free_storage_granted(&forest.forest_manifest())
            .unwrap())
    }

    #[rstest]
    fn test_creating_container(setup: (CargoUser, CatLibService, Forest)) {
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
        let path = "/some/path".to_owned();
        cargo_user
            .create_container(container_name.clone(), &storage_template, path)
            .unwrap();

        // then it is stored in catlib
        let retrieved_forest = catlib_service.get_forest(&forest.uuid()).unwrap();
        let containers = retrieved_forest.lock().unwrap().containers().unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!(
            containers[0].lock().unwrap().name().unwrap(),
            container_name
        );
        assert_eq!(
            containers[0]
                .lock()
                .unwrap()
                .forest()
                .unwrap()
                .lock()
                .unwrap()
                .uuid(),
            forest.uuid()
        );
    }

    #[rstest]
    fn test_getting_created_container(setup: (CargoUser, CatLibService, Forest)) {
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
        let path = "/some/path".to_owned();
        cargo_user
            .create_container(container_name.clone(), &storage_template, path)
            .unwrap();

        // then it can be retrieved via CargoUser api
        let containers = cargo_user.get_containers().unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].name().unwrap(), container_name);
    }

    #[rstest]
    fn test_delete_created_container(setup: (CargoUser, CatLibService, Forest)) {
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
        let path = "/some/path".to_owned();
        let container_name = "new container".to_string();
        let container = cargo_user
            .create_container(container_name, &storage_template, path)
            .unwrap();

        // and the container is deleted
        container.remove().unwrap();

        // then it cannot be retrieved via CargoUser api
        let containers = cargo_user.get_containers();
        assert!(matches!(containers, Err(CatlibError::NoRecordsFound)));

        // and the container handle received during creation is marked as deleted
        assert!(matches!(container.name(), Err(CatlibError::NoRecordsFound)));
    }
}
