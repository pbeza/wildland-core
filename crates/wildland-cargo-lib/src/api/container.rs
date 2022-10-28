use std::sync::{Arc, Mutex};

use crate::{
    api::{storage::Storage, storage_template::StorageTemplate},
    errors::{container::*, single_variant::*, storage::*},
};
use wildland_corex::{CatLibService, CatlibError, Container as InnerContainer, IContainer};

#[derive(Debug, Clone)]
pub struct Container {
    data: Arc<Mutex<ContainerData>>,
}

#[derive(Debug)]
pub struct ContainerData {
    inner: InnerContainer,
    // We cannot force a native app to drop reference to Container structure
    // so the flag is_deleted is used to mark container as deleted
    is_deleted: bool,
}

impl From<InnerContainer> for Container {
    fn from(inner: InnerContainer) -> Self {
        Self {
            data: Arc::new(Mutex::new(ContainerData {
                inner,
                is_deleted: false,
            })),
        }
    }
}

impl Container {
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn mount(&self) -> SingleErrVariantResult<(), ContainerMountError> {
        todo!()
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn unmount(&self) -> SingleErrVariantResult<(), ContainerUnmountError> {
        todo!()
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_storages(&self) -> SingleErrVariantResult<Vec<Storage>, GetStoragesError> {
        todo!()
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn delete_storage(
        &self,
        storage: &Storage,
    ) -> SingleErrVariantResult<(), DeleteStorageError> {
        todo!()
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn add_storage(
        &self,
        storage: &StorageTemplate,
    ) -> SingleErrVariantResult<(), AddStorageError> {
        todo!()
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn is_mounted(&self) -> bool {
        todo!()
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn set_name(&mut self, new_name: String) {
        let mut data = self.data.lock().expect("Could not lock containers data");
        data.inner.set_name(new_name)
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn stringify(&self) -> String {
        let data = self.data.lock().expect("Could not lock containers data");
        let deleted_info = if data.is_deleted { "DELETED: " } else { "" };
        let name = data.inner.name();
        format!("{deleted_info}Container (name: {name})")
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn duplicate(&self) -> SingleErrVariantResult<Container, CatlibError> {
        todo!()
    }

    pub fn delete(&self, catlib_service: &CatLibService) -> Result<(), CatlibError> {
        let mut data = self.data.lock().expect("Could not lock containers data");
        catlib_service.delete_container(&mut data.inner)?;
        data.is_deleted = true;
        Ok(())
    }
}
