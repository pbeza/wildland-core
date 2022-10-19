//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Create Container object from its representation in Rust Object Notation
impl TryFrom<String> for Container {
    type Error = ron::error::SpannedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ron::from_str(value.as_str())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Container {
    uuid: String,
    forest_uuid: String,
    paths: ContainerPaths,

    #[serde(skip, default = "use_default_database")]
    db: Rc<StoreDb>,
}

impl Container {
    pub fn new(forest_uuid: String, db: Rc<StoreDb>) -> Self {
        Container {
            uuid: Uuid::new_v4().to_string(),
            forest_uuid,
            db,
            paths: ContainerPaths::new(),
        }
    }
}

impl IContainer for Container {
    fn uuid(&self) -> String {
        self.uuid.clone()
    }

    fn forest(&self) -> CatlibResult<crate::forest::Forest> {
        fetch_forest_by_uuid(self.db.clone(), self.forest_uuid.clone())
    }

    fn paths(&self) -> ContainerPaths {
        self.paths.clone()
    }

    fn add_path(&mut self, path: ContainerPath) -> CatlibResult<Self> {
        self.paths.insert(path);
        self.save()?;
        Ok(self.clone())
    }

    fn del_path(&mut self, path: ContainerPath) -> CatlibResult<Self> {
        self.paths.remove(&path);
        self.save()?;
        Ok(self.clone())
    }

    fn storages(&self) -> CatlibResult<Vec<Storage>> {
        fetch_storages_by_container_uuid(self.db.clone(), self.uuid())
    }

    fn create_storage(
        &self,
        template_uuid: Option<String>,
        data: Vec<u8>,
    ) -> CatlibResult<Storage> {
        let mut storage = Storage::new(self.uuid(), template_uuid, data, self.db.clone());
        storage.save()?;

        Ok(storage)
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

        forest.create_container().unwrap()
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
            .create_storage(Some("template-1".into()), vec![])
            .unwrap();
        alpha
            .create_storage(Some("template-2".into()), vec![])
            .unwrap();

        beta.create_storage(Some("template-1".into()), vec![])
            .unwrap();

        let containers = catlib
            .find_containers_with_template("template-2".into())
            .unwrap();

        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].uuid(), alpha.uuid());

        let containers = catlib
            .find_containers_with_template("template-1".into())
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
        container
            .add_path("/bar/baz2".to_string())
            .unwrap()
            .add_path("/baz/qux1".to_string())
            .unwrap()
            .add_path("/baz/qux2".to_string())
            .unwrap();

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
