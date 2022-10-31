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

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wildland_catlib::{CatLib, CatlibError, Forest};
use wildland_crypto::identity::signing_keypair::PubKey;

use crate::WildlandIdentity;

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

#[derive(Clone)]
pub struct CatLibService {
    catlib: CatLib,
}

impl CatLibService {
    #[tracing::instrument(level = "debug")]
    pub fn new() -> Self {
        Self {
            catlib: CatLib::default(),
        }
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn add_forest(
        &self,
        forest_identity: &WildlandIdentity,
        this_device_identity: &WildlandIdentity,
        data: UserMetaData,
    ) -> Result<Forest, CatlibError> {
        self.catlib.create_forest(
            forest_identity.get_public_key().into(),
            HashSet::from([this_device_identity.get_public_key().into()]),
            serde_json::to_vec(&data)
                .map_err(|e| CatlibError::Generic(format!("Serialization error: {e}")))?,
        )
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_forest(&self, forest_uuid: Uuid) -> Result<Forest, CatlibError> {
        self.catlib.get_forest(forest_uuid)
    }
}