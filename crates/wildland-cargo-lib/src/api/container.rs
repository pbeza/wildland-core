use crate::{
    api::{storage::Storage, storage_template::StorageTemplate},
    errors::{container::*, single_variant::*, storage::*},
};
use uuid::Uuid;
use wildland_corex::{CatLibService, CatlibError, Container as InnerContainer, IContainer};

use super::cargo_user::SharedContainer;

#[derive(Debug)]
pub struct Container {
    inner: InnerContainer,
    // We cannot force a native app to drop reference to Container structure
    // so the flag is_deleted is used to mark container as deleted
    is_deleted: bool,
}

impl From<InnerContainer> for Container {
    fn from(inner: InnerContainer) -> Self {
        Self {
            inner,
            is_deleted: false,
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
        self.inner.set_name(new_name)
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn stringify(&self) -> String {
        let deleted_info = if self.is_deleted { "DELETED: " } else { "" };
        let name = self.inner.name();
        format!("{deleted_info}Container (name: {name})")
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn duplicate(&self) -> SingleErrVariantResult<SharedContainer, CatlibError> {
        todo!()
    }

    pub fn delete(&mut self, catlib_service: &CatLibService) -> Result<(), CatlibError> {
        catlib_service.delete_container(&mut self.inner)?;
        self.is_deleted = true;
        Ok(())
    }

    pub fn uuid(&self) -> Uuid {
        self.inner.uuid()
    }
}
