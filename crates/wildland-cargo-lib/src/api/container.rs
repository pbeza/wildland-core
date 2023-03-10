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

use uuid::Uuid;
use wildland_corex::catlib_service::entities::ContainerManifest as InnerContainer;
use wildland_corex::catlib_service::error::CatlibError;
use wildland_corex::catlib_service::CatLibService;

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
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    /// Returns string representation of a container
    pub fn stringify(&mut self) -> Result<String, CatlibError> {
        let deleted_info = if self.is_deleted { "DELETED: " } else { "" };
        let name = &self.inner.name()?;
        Ok(format!("{deleted_info}Container (name: {name})"))
    }

    /// Deletes Container Manifest related to this Container handle in CatLib and marks `self`
    /// object as deleted.
    pub fn delete(&mut self, catlib_service: &CatLibService) -> Result<(), CatlibError> {
        catlib_service.delete_container(self.inner.as_mut())?;
        self.is_deleted = true;
        Ok(())
    }

    /// Returns container uuid
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
        Ok(self.inner.paths().iter().cloned().collect())
    }

    pub fn get_name(&mut self) -> Result<String, CatlibError> {
        self.inner.name()
    }

    pub fn set_name(&mut self, new_name: String) {
        let _ = self.inner.set_name(new_name);
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use uuid::Uuid;
    use wildland_corex::catlib_service::{CatLibService, DeviceMetadata, ForestMetaData};
    use wildland_corex::{SigningKeypair, WildlandIdentity};

    use super::Container;
    use crate::api::container::tests::utils::compare_unordered;
    use crate::api::utils::test::catlib_service;
    use crate::templates::foundation_storage::FoundationStorageTemplate;

    #[fixture]
    fn container(catlib_service: CatLibService) -> Container {
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
                ForestMetaData::new(vec![DeviceMetadata {
                    name: dev_name,
                    pubkey: device_identity.get_public_key(),
                }]),
            )
            .unwrap();

        let fst = FoundationStorageTemplate::new(
            Uuid::new_v4(),
            "".to_owned(),
            "".to_owned(),
            "".to_owned(),
        )
        .try_into()
        .unwrap();

        Container {
            inner: catlib_service
                .create_container("name".to_owned(), forest.as_ref(), &fst)
                .unwrap(),
            is_deleted: false,
        }
    }

    #[rstest]
    fn test_name(mut container: Container) {
        assert_eq!(container.get_name().unwrap(), "name");
        container.set_name("new name".to_string());
        assert_eq!(container.get_name().unwrap(), "new name");
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
        use std::collections::HashSet;
        use std::hash::Hash;

        pub fn compare_unordered<T: Hash + Eq>(v1: Vec<T>, v2: Vec<T>) -> bool {
            v1.into_iter().collect::<HashSet<T>>() == v2.into_iter().collect::<HashSet<T>>()
        }
    }
}
