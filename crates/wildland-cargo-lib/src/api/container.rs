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

use crate::{
    api::{storage::Storage, storage_template::StorageTemplate},
    errors::{container::*, storage::*},
};
use uuid::Uuid;
use wildland_corex::catlib_service::{
    entities::Container as InnerContainer, error::CatlibError, CatLibService,
};

use super::cargo_user::SharedContainer;

/// Structure representing a container within Cargo application context.
///
/// It is designed to be safely used even in many threads on the native side (foreign languages) app.
#[derive(Debug)]
pub struct Container {
    /// CoreX container
    inner: Box<dyn InnerContainer>,
    // We cannot force a native app to drop reference to Container structure
    // so the flag is_deleted is used to mark container as deleted
    is_deleted: bool,
}

impl From<Box<dyn InnerContainer>> for Container {
    fn from(inner: Box<dyn InnerContainer>) -> Self {
        Self {
            inner,
            is_deleted: false,
        }
    }
}

impl Container {
    /// TODO
    pub fn mount(&self) -> Result<(), ContainerMountError> {
        todo!()
    }

    /// TODO
    pub fn unmount(&self) -> Result<(), ContainerUnmountError> {
        todo!()
    }

    /// TODO
    pub fn get_storages(&self) -> Result<Vec<Storage>, GetStoragesError> {
        todo!()
    }

    /// TODO
    pub fn delete_storage(&self, _storage: &Storage) -> Result<(), DeleteStorageError> {
        todo!()
    }

    /// TODO
    pub fn add_storage(&self, _storage: &StorageTemplate) -> Result<(), AddStorageError> {
        todo!()
    }

    /// TODO
    pub fn is_mounted(&self) -> bool {
        todo!()
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    /// Returns string representation of a container
    pub fn stringify(&self) -> String {
        let deleted_info = if self.is_deleted { "DELETED: " } else { "" };
        let name = &(*self.inner).as_ref().name;
        format!("{deleted_info}Container (name: {name})")
    }

    /// TODO
    pub fn duplicate(&self) -> Result<SharedContainer, CatlibError> {
        todo!()
    }

    /// TODO
    pub fn delete(&mut self, catlib_service: &CatLibService) -> Result<(), CatlibError> {
        catlib_service.delete_container(self.inner.as_mut())?;
        self.is_deleted = true;
        Ok(())
    }

    /// Returns container uuid
    pub fn uuid(&self) -> Uuid {
        (*self.inner).as_ref().uuid
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
        Ok((*self.inner).as_ref().paths.iter().cloned().collect())
    }

    pub fn get_name(&self) -> String {
        (*self.inner).as_ref().name.clone()
    }

    pub fn set_name(&mut self, new_name: String) {
        let _ = self.inner.set_name(new_name);
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::api::{
        container::tests::utils::compare_unordered, foundation_storage::FoundationStorageTemplate,
    };
    use rstest::*;
    use uuid::Uuid;
    use wildland_corex::{
        CatLibService, DeviceMetadata, ForestMetaData, SigningKeypair, WildlandIdentity,
    };

    use wildland_catlib::CatLib;

    use super::Container;

    #[fixture]
    fn container() -> Container {
        let catlib_service = CatLibService::new(Rc::new(CatLib::default()));

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
                ForestMetaData {
                    devices: vec![DeviceMetadata {
                        name: dev_name,
                        pubkey: device_identity.get_public_key(),
                    }],
                },
            )
            .unwrap();

        let fst = FoundationStorageTemplate {
            uuid: Uuid::new_v4(),
            credential_id: "".to_owned(),
            credential_secret: "".to_owned(),
            sc_url: "".to_owned(),
        };

        Container {
            inner: catlib_service
                .create_container("name".to_owned(), forest.as_ref(), &fst)
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
