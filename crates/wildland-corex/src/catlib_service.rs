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

use std::collections::HashSet;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wildland_crypto::identity::signing_keypair::PubKey;

use self::entities::{ContainerManifest, ForestManifest};
use self::error::{CatlibError, CatlibResult};
use self::interface::CatLib;
use crate::{StorageTemplate, TemplateContext, WildlandIdentity};

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
    ) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
        self.catlib.create_forest(
            forest_identity.get_public_key().into(),
            HashSet::from([this_device_identity.get_public_key().into()]),
            data.try_into()?,
        )
    }

    pub fn mark_free_storage_granted(
        &self,
        forest: &Arc<Mutex<dyn ForestManifest>>,
    ) -> CatlibResult<()> {
        let mut forest_metadata = self.get_parsed_forest_metadata(forest)?;
        forest_metadata.free_storage_granted = true;
        forest.lock().unwrap().update(forest_metadata.try_into()?)?;
        Ok(())
    }

    pub fn is_free_storage_granted(
        &self,
        forest: &Arc<Mutex<dyn ForestManifest>>,
    ) -> CatlibResult<bool> {
        let forest_metadata = self.get_parsed_forest_metadata(forest)?;
        Ok(forest_metadata.free_storage_granted)
    }

    pub fn get_forest(&self, forest_uuid: &Uuid) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
        self.catlib.get_forest(forest_uuid)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_container(
        &self,
        name: String,
        forest: &Arc<Mutex<dyn ForestManifest>>,
        storage_template: &StorageTemplate,
        path: ContainerPath,
    ) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>> {
        forest
            .lock()
            .expect("Poisoned Mutex")
            .create_container(name, storage_template, path)
    }

    pub fn delete_container(&self, container: &mut dyn ContainerManifest) -> CatlibResult<()> {
        container.delete().map(|_| ())
    }

    fn get_parsed_forest_metadata(
        &self,
        forest: &Arc<Mutex<dyn ForestManifest>>,
    ) -> CatlibResult<ForestMetaData> {
        serde_json::from_slice(&forest.lock().unwrap().data()?)
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
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;

    use mockall::predicate;
    use rstest::rstest;

    use crate::catlib_service::entities::{Identity, MockContainerManifest, MockStorageManifest};
    use crate::catlib_service::interface::MockCatLib;
    use crate::catlib_service::CatLibService;
    use crate::test_utils::MockForest;
    use crate::StorageTemplate;

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
            StorageTemplate::try_new("FoundationStorage", hashmap_template).unwrap();

        let mut forest_mock = MockForestManifest::new();
        forest_mock
            .expect_create_container()
            .with(
                predicate::eq(container_name.clone()),
                predicate::always(),
                predicate::eq("/some/path".to_owned()),
            )
            .times(1)
            .returning(move |_, _, _| Ok(Arc::new(Mutex::new(MockContainerManifest::new()))));

        let forest_mock: Arc<Mutex<dyn ForestManifest>> = Arc::new(Mutex::new(forest_mock));
        let path = "/some/path".to_owned();
        catlib_service
            .create_container(container_name, &forest_mock, &storage_template, path)
            .unwrap();
    }
}
