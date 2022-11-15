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
use std::rc::Rc;

use wildland_corex::entities::{
    Container as IContainer, ContainerData, ContainerPath, ContainerPaths, Forest,
    Storage as IStorage,
};

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct Container {
    data: ContainerData,

    #[derivative(Debug = "ignore")]
    db: Rc<StoreDb>,
}

impl AsRef<ContainerData> for Container {
    fn as_ref(&self) -> &ContainerData {
        &self.data
    }
}

impl Container {
    pub fn new(forest_uuid: Uuid, name: String, db: Rc<StoreDb>) -> Self {
        Self {
            data: ContainerData {
                uuid: Uuid::new_v4(),
                forest_uuid,
                name,
                paths: ContainerPaths::new(),
            },
            db,
        }
    }

    pub fn from_db_entry(value: &str, db: Rc<StoreDb>) -> Self {
        let data = ron::from_str(value).unwrap();
        Self { data, db }
    }
}

impl IContainer for Container {
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Forest`] was found.
    fn forest(&self) -> CatlibResult<Box<dyn Forest>> {
        fetch_forest_by_uuid(self.db.clone(), &self.data.forest_uuid)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::interface::CatLib as ICatLib;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let mut container = forest.create_container("container name".to_owned()).unwrap();
    /// container.add_path("/bar/baz2".to_string()).unwrap();
    /// ```
    fn add_path(&mut self, path: ContainerPath) -> CatlibResult<bool> {
        let inserted = self.data.paths.insert(path);
        self.save()?;
        Ok(inserted)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::interface::CatLib as ICatLib;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let mut container = forest.create_container("container name".to_owned()).unwrap();
    /// container.del_path("/baz/qux1".to_string()).unwrap();
    /// ```
    fn del_path(&mut self, path: ContainerPath) -> CatlibResult<bool> {
        let removed = self.data.paths.remove(&path);
        self.save()?;
        Ok(removed)
    }

    /// ## Errors
    ///
    /// Returns [`CatlibError::NoRecordsFound`] if Forest has no [`Storage`].
    fn storages(&self) -> CatlibResult<Vec<Box<dyn IStorage>>> {
        fetch_storages_by_container_uuid(self.db.clone(), &self.data.uuid)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::interface::CatLib as ICatLib;
    /// # use uuid::Uuid;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let mut container = forest.create_container("container name".to_owned()).unwrap();
    /// container.add_path("/foo/bar".to_string());
    /// container.create_storage(Some(Uuid::from_u128(1)), vec![]).unwrap();
    /// ```
    fn create_storage(
        &self,
        template_uuid: Option<Uuid>,
        data: Vec<u8>,
    ) -> CatlibResult<Box<dyn IStorage>> {
        let mut storage = Box::new(Storage::new(
            self.data.uuid,
            template_uuid,
            data,
            self.db.clone(),
        ));
        storage.save()?;

        Ok(storage)
    }

    fn set_name(&mut self, new_name: String) -> CatlibResult<()> {
        self.data.name = new_name;
        self.save()
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn delete(&mut self) -> CatlibResult<bool> {
        Model::delete(self)?;
        Ok(true)
    }
}

impl Model for Container {
    fn save(&mut self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("container-{}", self.data.uuid),
            ron::to_string(&self.data).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(self.db.clone(), format!("container-{}", self.data.uuid))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::db::test::catlib;
    use crate::*;
    use rstest::*;
    use wildland_corex::entities::Container;

    #[fixture]
    fn catlib_with_forest(catlib: CatLib) -> CatLib {
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

    fn make_container(catlib: &CatLib) -> Box<dyn Container> {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        forest.create_container("name".to_owned()).unwrap()
    }

    #[rstest(catlib_with_forest as catlib)]
    fn fetch_created_container(catlib: CatLib) {
        let container = make_container(&catlib);
        let container = catlib.get_container(&(*container).as_ref().uuid).unwrap();

        assert_eq!(
            (*container.forest().unwrap()).as_ref().owner,
            Identity([1; 32])
        );
    }

    #[rstest(catlib_with_forest as catlib)]
    fn fetch_created_container_from_forest_obj(catlib: CatLib) {
        let container = make_container(&catlib);
        let container = catlib.get_container(&(*container).as_ref().uuid).unwrap();

        assert_eq!(
            (*container.forest().unwrap()).as_ref().owner,
            Identity([1; 32])
        );
    }

    #[rstest(catlib_with_forest as catlib)]
    fn container_with_paths(catlib: CatLib) {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        let mut container = make_container(&catlib);
        container.add_path("/foo/bar".to_string()).unwrap();
        container.add_path("/bar/baz".to_string()).unwrap();

        assert!((*container)
            .as_ref()
            .paths
            .contains(&"/foo/bar".to_string()));
        assert!((*container)
            .as_ref()
            .paths
            .contains(&"/bar/baz".to_string()));

        // Try to find that container in the database
        let containers = forest
            .find_containers(vec!["/foo/bar".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);

        // Ensure again that it still has the paths
        assert!((*container)
            .as_ref()
            .paths
            .contains(&"/foo/bar".to_string()));
        assert!((*container)
            .as_ref()
            .paths
            .contains(&"/bar/baz".to_string()));

        // Try to fetch the same (one) container, using two different paths. The result
        // should be only one (not two) containers.
        let containers = forest
            .find_containers(vec!["/foo/bar".to_string(), "/bar/baz".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);
    }

    #[rstest(catlib_with_forest as catlib)]
    fn multiple_containers_with_paths(catlib: CatLib) {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

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
        assert_ne!(
            (*containers[0]).as_ref().uuid,
            (*containers[1]).as_ref().uuid
        );
    }

    #[rstest(catlib_with_forest as catlib)]
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
            .find_containers_with_template(&Uuid::from_u128(2))
            .unwrap();

        assert_eq!(containers.len(), 1);
        assert_eq!((*containers[0]).as_ref().uuid, (*alpha).as_ref().uuid);

        let containers = catlib
            .find_containers_with_template(&Uuid::from_u128(1))
            .unwrap();

        assert_eq!(containers.len(), 2);
        assert_ne!(
            (*containers[0]).as_ref().uuid,
            (*containers[1]).as_ref().uuid
        );
    }

    #[rstest(catlib_with_forest as catlib)]
    fn multiple_containers_with_subpaths(catlib: CatLib) {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

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
