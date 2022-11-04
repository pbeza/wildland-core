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

use super::*;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Create Container object from its representation in Rust Object Notation
impl TryFrom<&str> for Container {
    type Error = ron::error::SpannedError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ron::from_str(value)
    }
}

#[derive(Clone, Serialize, Deserialize, Derivative)]
#[derivative(Debug)]
pub struct Container {
    uuid: Uuid,
    forest_uuid: Uuid,
    name: String,
    paths: ContainerPaths,

    #[derivative(Debug = "ignore")]
    #[serde(skip, default = "use_default_database")]
    db: Rc<StoreDb>,
}

impl Container {
    pub fn new(forest_uuid: Uuid, name: String, db: Rc<StoreDb>) -> Self {
        Container {
            uuid: Uuid::new_v4(),
            forest_uuid,
            name,
            db,
            paths: ContainerPaths::new(),
        }
    }
}

impl IContainer for Container {
    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn forest(&self) -> CatlibResult<crate::forest::Forest> {
        fetch_forest_by_uuid(self.db.clone(), self.forest_uuid)
    }

    fn paths(&self) -> ContainerPaths {
        self.paths.clone()
    }

    /// Returns true if path was actually added, false otherwise.
    fn add_path(&mut self, path: ContainerPath) -> CatlibResult<bool> {
        let inserted = self.paths.insert(path);
        self.save()?;
        Ok(inserted)
    }

    /// Returns true if path was actually deleted, false otherwise.
    fn del_path(&mut self, path: ContainerPath) -> CatlibResult<bool> {
        let removed = self.paths.remove(&path);
        self.save()?;
        Ok(removed)
    }

    fn storages(&self) -> CatlibResult<Vec<Storage>> {
        fetch_storages_by_container_uuid(self.db.clone(), self.uuid())
    }

    fn create_storage(&self, template_uuid: Option<Uuid>, data: Vec<u8>) -> CatlibResult<Storage> {
        let mut storage = Storage::new(self.uuid(), template_uuid, data, self.db.clone());
        storage.save()?;

        Ok(storage)
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }
}

impl Model for Container {
    fn save(&mut self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("container-{}", self.uuid),
            ron::to_string(self).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(self.db.clone(), format!("container-{}", self.uuid))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::contracts::IContainer;
    use crate::*;
    use rstest::*;
    use uuid::Bytes;

    #[fixture]
    fn catlib() -> CatLib {
        let catlib = db::init_catlib(rand::random::<Bytes>());

        // Create a dummy forest to which containers will be bound
        catlib
            .create_forest(
                Identity([1; 32]),
                HashSet::from([Identity([2; 32])]),
                vec![],
            )
            .unwrap();

        catlib
    }

    fn make_container(catlib: &CatLib) -> crate::container::Container {
        let forest = catlib.find_forest(Identity([1; 32])).unwrap();

        forest.create_container("name".to_owned()).unwrap()
    }

    #[rstest]
    fn fetch_created_container(catlib: CatLib) {
        let container = make_container(&catlib);
        let container = catlib.get_container(container.uuid()).unwrap();

        assert_eq!(container.forest().unwrap().owner(), Identity([1; 32]));
    }

    #[rstest]
    fn fetch_created_container_from_forest_obj(catlib: CatLib) {
        let container = make_container(&catlib);
        let container = catlib.get_container(container.uuid()).unwrap();

        assert_eq!(container.forest().unwrap().owner(), Identity([1; 32]));
    }

    #[rstest]
    fn container_with_paths(catlib: CatLib) {
        let forest = catlib.find_forest(Identity([1; 32])).unwrap();

        let mut container = make_container(&catlib);
        container.add_path("/foo/bar".to_string()).unwrap();
        container.add_path("/bar/baz".to_string()).unwrap();

        assert!(container.paths().contains(&"/foo/bar".to_string()));
        assert!(container.paths().contains(&"/bar/baz".to_string()));

        // Try to find that container in the database
        let containers = forest
            .find_containers(vec!["/foo/bar".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);

        // Ensure again that it still has the paths
        assert!(container.paths().contains(&"/foo/bar".to_string()));
        assert!(container.paths().contains(&"/bar/baz".to_string()));

        // Try to fetch the same (one) container, using two different paths. The result
        // should be only one (not two) containers.
        let containers = forest
            .find_containers(vec!["/foo/bar".to_string(), "/bar/baz".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);
    }

    #[rstest]
    fn multiple_containers_with_paths(catlib: CatLib) {
        let forest = catlib.find_forest(Identity([1; 32])).unwrap();

        let mut container = make_container(&catlib);
        container.add_path("/foo/bar".to_string()).unwrap();
        container.add_path("/bar/baz".to_string()).unwrap();

        // Create another container, that shares a path with the former
        let mut container = make_container(&catlib);
        container.add_path("/bar/baz".to_string()).unwrap();

        // And yet another container that doesn't
        let mut container = make_container(&catlib);
        container.add_path("/what/ever".to_string()).unwrap();

        // try to find the first container
        let containers = forest
            .find_containers(vec!["/foo/bar".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);

        // try to find the first and the second containers, using shared path
        let containers = forest
            .find_containers(vec!["/bar/baz".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 2);

        // Make sure that they are in fact two different containers
        assert_ne!(containers[0].uuid(), containers[1].uuid());
    }

    #[rstest]
    fn create_containers_with_different_storages(catlib: CatLib) {
        let alpha = make_container(&catlib);
        let beta = make_container(&catlib);

        alpha
            .create_storage(Some(Uuid::from_u128(1)), vec![])
            .unwrap();
        alpha
            .create_storage(Some(Uuid::from_u128(2)), vec![])
            .unwrap();

        beta.create_storage(Some(Uuid::from_u128(1)), vec![])
            .unwrap();

        let containers = catlib
            .find_containers_with_template(Uuid::from_u128(2))
            .unwrap();

        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].uuid(), alpha.uuid());

        let containers = catlib
            .find_containers_with_template(Uuid::from_u128(1))
            .unwrap();

        assert_eq!(containers.len(), 2);
        assert_ne!(containers[0].uuid(), containers[1].uuid());
    }

    #[rstest]
    fn multiple_containers_with_subpaths(catlib: CatLib) {
        let forest = catlib.find_forest(Identity([1; 32])).unwrap();

        let mut container = make_container(&catlib);
        container.add_path("/foo/bar1".to_string()).unwrap();

        let mut container = make_container(&catlib);
        container.add_path("/foo/bar2".to_string()).unwrap();
        container.add_path("/bar/baz1".to_string()).unwrap();

        let mut container = make_container(&catlib);
        container.add_path("/bar/baz2".to_string()).unwrap();
        container.add_path("/baz/qux1".to_string()).unwrap();
        container.add_path("/baz/qux2".to_string()).unwrap();

        let containers = forest.find_containers(vec!["/foo".into()], false);
        assert_eq!(containers.err(), Some(CatlibError::NoRecordsFound));

        let containers = forest
            .find_containers(vec!["/foo/bar1".into()], true)
            .unwrap();
        assert_eq!(containers.len(), 1);

        let containers = forest.find_containers(vec!["/foo".into()], true).unwrap();
        assert_eq!(containers.len(), 2);

        let containers = forest.find_containers(vec!["/bar".into()], true).unwrap();
        assert_eq!(containers.len(), 2);

        let containers = forest.find_containers(vec!["/baz".into()], true).unwrap();
        assert_eq!(containers.len(), 1);

        let containers = forest.find_containers(vec!["/".into()], true).unwrap();
        assert_eq!(containers.len(), 3);
    }
}
