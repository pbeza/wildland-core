use crate::{
    api::storage_template::*,
    errors::{single_variant::*, user::*},
};
use derivative::Derivative;
use wildland_corex::{CatLibService, CatlibError, Forest, IForest};

use super::container::Container;

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct CargoUser {
    this_device: String,
    all_devices: Vec<String>,

    forest: Forest,

    #[derivative(Debug = "ignore")]
    catlib_service: CatLibService,
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
    pub fn get_containers(&self) -> SingleErrVariantResult<Vec<Container>, CatlibError> {
        self.forest
            .containers()
            .map_err(SingleVariantError::Failure)
            .map(|inner| {
                inner
                    .into_iter()
                    .map(|c| c.into())
                    .collect::<Vec<Container>>()
            })
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn create_container(
        &self,
        name: String,
        template: &StorageTemplate,
    ) -> SingleErrVariantResult<Container, CatlibError> {
        self.catlib_service
            .create_container(name, &self.forest, template.inner())
            .map_err(SingleVariantError::Failure)
            .map(|c| c.into())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn delete_container(
        &self,
        container: &Container,
    ) -> SingleErrVariantResult<(), CatlibError> {
        container
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
