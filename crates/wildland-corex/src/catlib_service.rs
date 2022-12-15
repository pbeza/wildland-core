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

pub mod entities;
pub mod error;
pub mod interface;

use std::{collections::HashSet, rc::Rc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wildland_crypto::identity::signing_keypair::PubKey;

use crate::{StorageTemplate, TemplateContext, WildlandIdentity};

use self::{
    entities::{ContainerManifest, ForestManifest},
    error::{CatlibError, CatlibResult},
    interface::CatLib,
};

#[derive(Serialize, Deserialize)]
pub struct DeviceMetadata {
    pub name: String,
    pub pubkey: PubKey,
}

#[derive(Serialize, Deserialize)]
pub struct ForestMetaData {
    devices: Vec<DeviceMetadata>,
    free_storage_granted: bool,
}

impl ForestMetaData {
    pub fn new(devices: Vec<DeviceMetadata>) -> Self {
        Self {
            devices,
            free_storage_granted: false,
        }
    }

    pub fn get_device_metadata(&self, device_pubkey: PubKey) -> Option<&DeviceMetadata> {
        self.devices.iter().find(|d| d.pubkey == device_pubkey)
    }

    pub fn devices(&self) -> impl Iterator<Item = &DeviceMetadata> {
        self.devices.iter()
    }
}

impl TryFrom<ForestMetaData> for Vec<u8> {
    type Error = CatlibError;

    fn try_from(data: ForestMetaData) -> Result<Self, Self::Error> {
        serde_json::to_vec(&data)
            .map_err(|e| CatlibError::Generic(format!("Serialization error: {e}")))
    }
}

#[derive(Clone)]
pub struct CatLibService {
    catlib: Rc<dyn CatLib>,
}

impl CatLibService {
    pub fn new(catlib: Rc<dyn CatLib>) -> Self {
        Self { catlib }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn add_forest(
        &self,
        forest_identity: &WildlandIdentity,
        this_device_identity: &WildlandIdentity,
        data: ForestMetaData,
    ) -> CatlibResult<Box<dyn ForestManifest>> {
        self.catlib.create_forest(
            forest_identity.get_public_key().into(),
            HashSet::from([this_device_identity.get_public_key().into()]),
            data.try_into()?,
        )
    }

    pub fn mark_free_storage_granted(&self, forest: &mut dyn ForestManifest) -> CatlibResult<()> {
        let mut forest_metadata = self.get_parsed_forest_metadata(forest)?;
        forest_metadata.free_storage_granted = true;
        forest.update(forest_metadata.try_into()?)?;
        Ok(())
    }

    pub fn is_free_storage_granted(&self, forest: &mut dyn ForestManifest) -> CatlibResult<bool> {
        let forest_metadata = self.get_parsed_forest_metadata(forest)?;
        Ok(forest_metadata.free_storage_granted)
    }

    pub fn get_forest(&self, forest_uuid: &Uuid) -> CatlibResult<Box<dyn ForestManifest>> {
        self.catlib.get_forest(forest_uuid)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_container(
        &self,
        name: String,
        forest: &dyn ForestManifest,
        storage_template: &StorageTemplate,
    ) -> CatlibResult<Box<dyn ContainerManifest>> {
        let container = forest.create_container(name.clone())?;

        // TODO access mode
        // TODO other params
        let template_context = TemplateContext {
            container_name: name,
            owner: forest.owner().encode(),
            access_mode: crate::StorageAccessMode::ReadWrite,
        };
        let storage = storage_template.render(template_context).unwrap(); // TODO unwrap

        let serialized_storage = serde_json::to_vec(&storage).map_err(|e| {
            CatlibError::Generic(format!("Could not serialize storage template: {e}"))
        })?;

        let _storage = container.create_storage(Some(storage.uuid()), serialized_storage)?;

        Ok(container)
    }

    pub fn delete_container(&self, container: &mut dyn ContainerManifest) -> CatlibResult<()> {
        container.delete().map(|_| ())
    }

    fn get_parsed_forest_metadata(
        &self,
        forest: &mut dyn ForestManifest,
    ) -> CatlibResult<ForestMetaData> {
        serde_json::from_slice(&forest.data()?)
            .map_err(|e| CatlibError::Generic(format!("Could not deserialize forest metadata {e}")))
    }

    pub fn get_storage_templates_data(&self) -> CatlibResult<Vec<String>> {
        self.catlib.get_storage_templates_data()
    }

    pub fn save_storage_template(&self, storage_template: &StorageTemplate) -> CatlibResult<()> {
        self.catlib.save_storage_template(
            &storage_template.uuid(),
            serde_json::to_string(storage_template)
                .map_err(|e| CatlibError::Generic(format!("Could not serialize object: {e}")))?,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, rc::Rc};

    use mockall::{mock, predicate};
    use rstest::rstest;
    use serde_json::json;
    use uuid::Uuid;

    use crate::{
        entities::{
            Bridge, ContainerManifest, ContainerPath, ForestManifest, Identity,
            MockContainerManifest, MockStorageManifest, Signers,
        },
        interface::MockCatLib,
        CatLibService, StorageBackendType, StorageTemplate,
    };

    use super::error::CatlibResult;

    mock! {
        #[derive(Debug)]
        pub Forest {}
        impl Clone for Forest {
            fn clone(&self) -> Self;
        }
        impl ForestManifest for Forest {
            fn add_signer(&mut self, signer: Identity) -> CatlibResult<bool>;
            fn del_signer(&mut self, signer: Identity) -> CatlibResult<bool>;
            fn containers(&self) -> CatlibResult<Vec<Box<dyn ContainerManifest>>>;
            fn update(&mut self, data: Vec<u8>) -> CatlibResult<()>;
            fn delete(&mut self) -> CatlibResult<bool>;
            fn create_container(&self, name: String) -> CatlibResult<Box<dyn ContainerManifest>>;
            fn create_bridge( &self,
                path: ContainerPath,
                link_data: Vec<u8>,
            ) -> CatlibResult<Box<dyn Bridge>>;
            fn find_bridge(&self, path: ContainerPath) -> CatlibResult<Box<dyn Bridge>>;
            fn find_containers( &self,
                paths: Vec<ContainerPath>,
                include_subdirs: bool,
            ) -> CatlibResult<Vec<Box<dyn ContainerManifest>>>;
            fn data(&mut self) -> CatlibResult<Vec<u8>>;
            fn uuid(&self) -> Uuid;
            fn owner(&self) -> Identity;
            fn signers(&mut self) -> CatlibResult<Signers>;

        }
    }

    #[rstest]
    fn test_create_container() {
        let catlib_mock = MockCatLib::new();
        let catlib = Rc::new(catlib_mock);
        let catlib_service = CatLibService::new(catlib);

        let container_name = "Books".to_owned();

        let hashmap_template = HashMap::from([
            ("field1", "prefix {{ OWNER }} suffix"),
            ("field2", "{{ CONTAINER_NAME }}"),
        ]);
        let storage_template =
            StorageTemplate::try_new(StorageBackendType::FoundationStorage, hashmap_template)
                .unwrap();

        let mut forest_mock = MockForest::new();
        forest_mock
            .expect_create_container()
            .with(predicate::eq(container_name.clone()))
            .times(1)
            .returning(move |name| {
                let mut container_mock = MockContainerManifest::new();
                let expected_storage_json = json!({
                    "name": null,
                    "uuid": null, // avoid comparing random value
                    "backend_type": "FoundationStorage",
                    "data": {
                        "field1": "prefix 0101010101010101010101010101010101010101010101010101010101010101 suffix",
                        "field2": name
                    }
                });
                container_mock
                    .expect_create_storage()
                    .withf(move |_uuid, bytes|{
                        let mut json: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                        json["uuid"] = serde_json::Value::Null; // avoid comparing random value
                        json == expected_storage_json
                    })
                    .times(1)
                    .returning(|_, _|Ok(Box::new(MockStorageManifest::new())));
                Ok(Box::new(container_mock))
            });
        forest_mock
            .expect_owner()
            .times(1)
            .returning(|| Identity([1; 32]));

        catlib_service
            .create_container(container_name, &forest_mock, &storage_template)
            .unwrap();
    }
}
