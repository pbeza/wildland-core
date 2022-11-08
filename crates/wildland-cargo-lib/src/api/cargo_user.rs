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
        self.forest.containers().map(|inner| {
            inner
                .into_iter()
                .map(|c| {
                    let container = Container::from(c);
                    let uuid = container.uuid();
                    if let Some(cached_container) = self.user_context.get_loaded_container(uuid) {
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
                .collect::<Vec<SharedContainer>>()
        })
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
