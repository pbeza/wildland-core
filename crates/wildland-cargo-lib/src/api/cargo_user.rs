use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    api::storage_template::*,
    errors::{single_variant::*, user::*},
};
use derivative::Derivative;
use uuid::Uuid;
use wildland_corex::{CatLibService, CatlibError, Forest, IForest};

use super::container::Container;

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

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct CargoUser {
    this_device: String,
    all_devices: Vec<String>,

    forest: Forest,

    #[derivative(Debug = "ignore")]
    catlib_service: CatLibService,
    #[derivative(Debug = "ignore")]
    user_context: UserContext,
}

impl CargoUser {
    pub fn new(
        this_device: String,
        all_devices: Vec<String>,
        forest: Forest,
        catlib_service: CatLibService,
    ) -> Self {
        Self {
            this_device,
            all_devices,
            forest,
            catlib_service,
            user_context: UserContext::new(),
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
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

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn mount_forest(&self) -> SingleErrVariantResult<(), ForestMountError> {
        todo!()
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_containers(
        &self,
    ) -> SingleErrVariantResult<Vec<Arc<Mutex<Container>>>, CatlibError> {
        self.forest
            .containers()
            .map_err(SingleVariantError::Failure)
            .map(|inner| {
                inner
                    .into_iter()
                    .map(|c| {
                        let container = Container::from(c);
                        let uuid = container.uuid();
                        self.user_context
                            .get_loaded_container(uuid) // Return container from user context if it has been already loaded
                            .unwrap_or_else(|| {
                                // or create one in context and return
                                let shared = Arc::new(Mutex::new(container));
                                self.user_context.add_container(uuid, shared.clone());
                                shared
                            })
                    })
                    .collect::<Vec<SharedContainer>>()
            })
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn create_container(
        &self,
        name: String,
        template: &StorageTemplate,
    ) -> SingleErrVariantResult<SharedContainer, CatlibError> {
        let container: Container = self
            .catlib_service
            .create_container(name, &self.forest, template.inner())
            .map_err(SingleVariantError::Failure)
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
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn delete_container(
        &self,
        container: &SharedContainer,
    ) -> SingleErrVariantResult<(), CatlibError> {
        container
            .lock()
            .expect("Could not lock shared container while deleting")
            .delete(&self.catlib_service)
            .map_err(SingleVariantError::Failure)
    }

    pub fn this_device(&self) -> &str {
        &self.this_device
    }

    pub fn all_devices(&self) -> &[String] {
        self.all_devices.as_slice()
    }
}
