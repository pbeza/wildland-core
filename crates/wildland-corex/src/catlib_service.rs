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

use self::entities::Forest;
use self::error::{CatlibError, CatlibResult};
use self::interface::CatLib;

#[derive(Serialize, Deserialize)]
pub struct DeviceMetadata {
    pub name: String,
    pub pubkey: PubKey,
}

#[derive(Serialize, Deserialize)]
pub struct UserMetaData {
    pub devices: Vec<DeviceMetadata>,
}

impl UserMetaData {
    pub fn get_device_metadata(&self, device_pubkey: PubKey) -> Option<&DeviceMetadata> {
        self.devices.iter().find(|d| d.pubkey == device_pubkey)
    }
}

#[derive(Clone, Default)]
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
        data: UserMetaData,
    ) -> CatlibResult<Box<dyn Forest>> {
        self.catlib.create_forest(
            forest_identity.get_public_key().into(),
            HashSet::from([this_device_identity.get_public_key().into()]),
            serde_json::to_vec(&data)
                .map_err(|e| CatlibError::Generic(format!("Serialization error: {e}")))?,
        )
    }

    pub fn get_forest(&self, forest_uuid: &Uuid) -> CatlibResult<Box<dyn Forest>> {
        self.catlib.get_forest(forest_uuid)
    }

    pub fn create_container(
        &self,
        name: String,
        forest: &Forest,
        storage_template: &impl StorageTemplateTrait,
    ) -> Result<Container, CatlibError> {
        let container = forest.create_container(name)?;

        let serialized_storage_template = serde_json::to_vec(&storage_template).map_err(|e| {
            CatlibError::Generic(format!("Could not serialize storage template: {e}"))
        })?;

        let _storage =
            container.create_storage(Some(storage_template.uuid()), serialized_storage_template)?;

        Ok(container)
    }

    pub fn delete_container(&self, container: &mut Container) -> Result<(), CatlibError> {
        container.delete()
    }
}
