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

use crate::{storage::StorageTemplateTrait, WildlandIdentity};

use self::entities::{Container, Forest};
use self::error::{CatlibError, CatlibResult};
use self::interface::CatLib;

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

    pub fn add_forest(
        &self,
        forest_identity: &WildlandIdentity,
        this_device_identity: &WildlandIdentity,
        data: ForestMetaData,
    ) -> CatlibResult<Box<dyn Forest>> {
        self.catlib.create_forest(
            forest_identity.get_public_key().into(),
            HashSet::from([this_device_identity.get_public_key().into()]),
            data.try_into()?,
        )
    }

    pub fn mark_free_storage_granted(&self, forest: &mut Box<dyn Forest>) -> CatlibResult<()> {
        let mut forest_metadata = self.get_parsed_forest_metadata(forest.as_ref())?;
        forest_metadata.free_storage_granted = true;
        forest.as_mut().update(forest_metadata.try_into()?)?;
        Ok(())
    }

    pub fn is_free_storage_granted(&self, forest: &dyn Forest) -> CatlibResult<bool> {
        let forest_metadata = self.get_parsed_forest_metadata(forest)?;
        Ok(forest_metadata.free_storage_granted)
    }

    pub fn get_forest(&self, forest_uuid: &Uuid) -> CatlibResult<Box<dyn Forest>> {
        self.catlib.get_forest(forest_uuid)
    }

    pub fn create_container(
        &self,
        name: String,
        forest: &dyn Forest,
        storage_template: &impl StorageTemplateTrait,
    ) -> CatlibResult<Box<dyn Container>> {
        let container = forest.create_container(name)?;

        let serialized_storage_template = serde_json::to_vec(&storage_template).map_err(|e| {
            CatlibError::Generic(format!("Could not serialize storage template: {e}"))
        })?;

        let _storage =
            container.create_storage(Some(storage_template.uuid()), serialized_storage_template)?;

        Ok(container)
    }

    pub fn delete_container(&self, container: &mut dyn Container) -> CatlibResult<()> {
        container.delete().map(|_| ())
    }

    fn get_parsed_forest_metadata(&self, forest: &dyn Forest) -> CatlibResult<ForestMetaData> {
        serde_json::from_slice(&forest.as_ref().data)
            .map_err(|e| CatlibError::Generic(format!("Could not deserialize forest metadata {e}")))
    }
}
