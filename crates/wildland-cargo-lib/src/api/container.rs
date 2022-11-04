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
    pub fn mount(&self) -> SingleErrVariantResult<(), ContainerMountError> {
        todo!()
    }

    pub fn unmount(&self) -> SingleErrVariantResult<(), ContainerUnmountError> {
        todo!()
    }

    pub fn get_storages(&self) -> SingleErrVariantResult<Vec<Storage>, GetStoragesError> {
        todo!()
    }

    pub fn delete_storage(
        &self,
        _storage: &Storage,
    ) -> SingleErrVariantResult<(), DeleteStorageError> {
        todo!()
    }

    pub fn add_storage(
        &self,
        _storage: &StorageTemplate,
    ) -> SingleErrVariantResult<(), AddStorageError> {
        todo!()
    }

    pub fn is_mounted(&self) -> bool {
        todo!()
    }

    pub fn stringify(&self) -> String {
        let deleted_info = if self.is_deleted { "DELETED: " } else { "" };
        let name = self.inner.name();
        format!("{deleted_info}Container (name: {name})")
    }

    pub fn duplicate(&self) -> Result<SharedContainer, CatlibError> {
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

    /// Returns true if path was actually added, false otherwise.
    pub fn add_path(&mut self, path: String) -> Result<bool, CatlibError> {
        self.inner.add_path(path)
    }

    /// Returns true if path was actually deleted, false otherwise.
    pub fn delete_path(&mut self, path: String) -> Result<bool, CatlibError> {
        self.inner.del_path(path)
    }

    /// Returns paths in arbitrary order.
    pub fn get_paths(&self) -> Result<Vec<String>, CatlibError> {
        Ok(self.inner.paths().into_iter().collect())
    }

    pub fn get_name(&self) -> String {
        self.inner.name()
    }

    pub fn set_name(&mut self, new_name: String) {
        self.inner.set_name(new_name)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{
        container::tests::utils::compare_unordered, foundation_storage::FoundationStorageTemplate,
    };
    use rstest::*;
    use std::rc::Rc;
    use uuid::Uuid;
    use wildland_corex::{
        storage::StorageTemplate, CatLibService, DeviceMetadata, SigningKeypair, UserMetaData,
        WildlandIdentity,
    };

    use super::Container;

    #[fixture]
    fn container() -> Container {
        let catlib_service = CatLibService::new();

        let dev_name = "dev name".to_string();
        let forest_identity = WildlandIdentity::Forest(
            0,
            SigningKeypair::try_from_bytes_slices([1; 32], [2; 32]).unwrap(),
        );
        let device_identity = WildlandIdentity::Device(
            dev_name.clone(),
            SigningKeypair::try_from_bytes_slices([3; 32], [4; 32]).unwrap(),
        );

        let forest = catlib_service
            .add_forest(
                &forest_identity,
                &device_identity,
                UserMetaData {
                    devices: vec![DeviceMetadata {
                        name: dev_name,
                        pubkey: device_identity.get_public_key(),
                    }],
                },
            )
            .unwrap();

        let fst = StorageTemplate::new(Rc::new(FoundationStorageTemplate {
            id: Uuid::new_v4(),
            credential_id: "".to_owned(),
            credential_secret: "".to_owned(),
            sc_url: "".to_owned(),
        }));

        Container {
            inner: catlib_service
                .create_container("name".to_owned(), &forest, &fst)
                .unwrap(),
            is_deleted: false,
        }
    }

    #[rstest]
    fn test_name(mut container: Container) {
        assert_eq!(container.get_name(), "name");
        container.set_name("new name".to_string());
        assert_eq!(container.get_name(), "new name");
    }

    #[rstest]
    fn test_paths(mut container: Container) {
        let path1 = "/path/1".to_string();
        let path2 = "/path/2".to_string();

        assert!(compare_unordered(
            container.get_paths().unwrap(),
            Vec::<String>::new()
        ));

        assert!(container.add_path(path1.clone()).unwrap());
        assert!(container.add_path(path2.clone()).unwrap());
        assert!(compare_unordered(
            container.get_paths().unwrap(),
            vec![path1.clone(), path2.clone()]
        ));

        assert!(container.delete_path(path2.clone()).unwrap());
        assert!(compare_unordered(
            container.get_paths().unwrap(),
            vec![path1]
        ));

        assert!(!container.delete_path(path2).unwrap());
    }

    mod utils {
        use std::{collections::HashSet, hash::Hash};

        pub fn compare_unordered<T: Hash + Eq>(v1: Vec<T>, v2: Vec<T>) -> bool {
            v1.into_iter().collect::<HashSet<T>>() == v2.into_iter().collect::<HashSet<T>>()
        }
    }
}
