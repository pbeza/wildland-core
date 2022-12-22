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

use std::collections::HashSet;
use std::rc::Rc;

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use wildland_corex::catlib_service::entities::{
    ContainerManifest as IContainer,
    ContainerPath,
    ContainerPaths,
    ForestManifest,
    StorageManifest as IStorage,
};

use super::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ContainerData {
    pub uuid: Uuid,
    pub forest_uuid: Uuid,
    pub name: String,
    pub paths: ContainerPaths,
}

impl From<&str> for ContainerData {
    fn from(str_data: &str) -> Self {
        ron::from_str(str_data).unwrap()
    }
}

//
// TODO (tkulik): Delegate syncing to some other instance (periodical poll or by notification)
//
#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) struct Container {
    pub(crate) container_data: ContainerData,
    pub(crate) forest_owner: Arc<Mutex<dyn ForestManifest>>,
    pub(crate) storages: Vec<Arc<Mutex<dyn StorageManifest>>>,
    #[derivative(Debug = "ignore")]
    pub(crate) db: Rc<StoreDb>,
}

impl Container {
    pub fn new(
        forest_owner: Arc<Mutex<dyn ForestManifest>>,
        storage_template: &StorageTemplate,
        name: String,
        db: Rc<StoreDb>,
    ) -> Result<Self, CatlibError> {
        let container_uuid = Uuid::new_v4();
        let forest_uuid = forest_owner
            .lock()
            .map_err(|_| CatlibError::Generic("Poisoned Mutex".to_owned()))?
            .uuid();
        let container_data = ContainerData {
            uuid: container_uuid,
            forest_uuid,
            name,
            paths: ContainerPaths::new(),
        };
        let mut container = Self {
            container_data,
            forest_owner,
            storages: vec![],
            db,
        };
        container.save()?;
        container.add_storage(storage_template)?;
        Ok(container)
    }

    pub fn from_container_data(
        container_data: ContainerData,
        db: Rc<StoreDb>,
    ) -> Result<Self, CatlibError> {
        let container_uuid = container_data.uuid;
        let forest_uuid = container_data.forest_uuid;
        let forest_owner = fetch_forest_by_uuid(db.clone(), &forest_uuid)?;
        let storages = fetch_storages_by_container_uuid(db.clone(), &container_uuid)?;
        Ok(Self {
            container_data,
            forest_owner,
            storages,
            db,
        })
    }
}

impl ContainerManifest for Container {
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Forest`] was found.
    fn forest(&self) -> Result<Arc<Mutex<dyn ForestManifest>>, CatlibError> {
        Ok(self.forest_owner.clone())
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::{HashSet, HashMap};
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::interface::CatLib as ICatLib;
    /// # use wildland_corex::StorageTemplate;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let storage_template = StorageTemplate::try_new(
    ///     "FoundationStorage",
    ///     HashMap::from([
    ///             (
    ///                 "field1".to_owned(),
    ///                 "Some value with container name: {{ CONTAINER_NAME }}".to_owned(),
    ///             ),
    ///             (
    ///                 "parameter in key: {{ OWNER }}".to_owned(),
    ///                 "enum: {{ ACCESS_MODE }}".to_owned(),
    ///             ),
    ///             ("uuid".to_owned(), "{{ CONTAINER_UUID }}".to_owned()),
    ///             ("paths".to_owned(), "{{ PATHS }}".to_owned()),
    ///         ]),
    ///     )
    ///     .unwrap();
    /// let container = forest.lock().unwrap().create_container("container name2".to_owned(), &storage_template).unwrap();
    /// container.lock().unwrap().add_path("/bar/baz2".to_string()).unwrap();
    /// ```
    fn add_path(&mut self, path: ContainerPath) -> Result<bool, CatlibError> {
        self.sync()?;
        let inserted = self.container_data.paths.insert(path);
        self.save()?;
        Ok(inserted)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::{HashSet, HashMap};
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::interface::CatLib as ICatLib;
    /// # use wildland_corex::StorageTemplate;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let storage_template = StorageTemplate::try_new(
    ///     "FoundationStorage",
    ///     HashMap::from([
    ///             (
    ///                 "field1".to_owned(),
    ///                 "Some value with container name: {{ CONTAINER_NAME }}".to_owned(),
    ///             ),
    ///             (
    ///                 "parameter in key: {{ OWNER }}".to_owned(),
    ///                 "enum: {{ ACCESS_MODE }}".to_owned(),
    ///             ),
    ///             ("uuid".to_owned(), "{{ CONTAINER_UUID }}".to_owned()),
    ///             ("paths".to_owned(), "{{ PATHS }}".to_owned()),
    ///         ]),
    ///     )
    ///     .unwrap();
    /// let container = forest.lock().unwrap().create_container("container name2".to_owned(), &storage_template).unwrap();
    /// container.lock().unwrap().delete_path("/baz/qux1".to_string()).unwrap();
    /// ```
    fn delete_path(&mut self, path: ContainerPath) -> Result<bool, DeleteContainerPathError> {
        // TODO (tkulik): Error handling
        self.sync().map_err(|_| DeleteContainerPathError::Error)?;
        let removed = self.container_data.paths.remove(&path);
        self.save().map_err(|_| DeleteContainerPathError::Error)?;
        Ok(removed)
    }

    /// TODO (tkulik): Add docstring
    ///
    fn add_storage(&mut self, storage_template: &StorageTemplate) -> Result<(), CatlibError> {
        let template_context = TemplateContext {
            container_name: self.container_data.name.clone(),
            owner: self
                .forest_owner
                .lock()
                .map_err(|_| CatlibError::Generic("Poisoned Mutex".to_owned()))?
                .owner()
                .encode(),
            access_mode: wildland_corex::StorageAccessMode::ReadWrite,
            container_uuid: self.container_data.uuid,
            paths: ContainerPaths::new(), // TODO (tkulik): Add at least one path here.
        };
        let storage = storage_template
            .render(template_context)
            .map_err(|e| CatlibError::Generic(e.to_string()))?;
        let serialized_storage = serde_json::to_vec(&storage).map_err(|e| {
            CatlibError::Generic(format!("Could not serialize storage template: {e}"))
        })?;
        let storage_entity = StorageEntity::new(
            self.container_data.uuid,
            Some(storage.uuid()),
            serialized_storage,
            self.db.clone(),
        );
        storage_entity.save()?;
        self.sync()?;
        self.storages.push(Arc::new(Mutex::new(storage_entity)));
        Ok(())
    }

    /// TODO (tkulik): Add docstring
    ///
    fn is_deleted(&mut self) -> bool {
        matches!(self.sync(), Err(CatlibError::NoRecordsFound))
    }

    /// ## Errors
    ///
    /// Returns [`CatlibError::NoRecordsFound`] if Forest has no [`Storage`].
    fn get_storages(&mut self) -> Result<Vec<Arc<Mutex<dyn StorageManifest>>>, CatlibError> {
        self.sync()?;
        Ok(self.storages.clone())
    }

    /// TODO (tkulik): Add docstring
    ///
    fn stringify(&self) -> String {
        format!("{self:?}")
    }

    fn set_name(&mut self, new_name: String) -> CatlibResult<()> {
        self.sync()?;
        self.container_data.name = new_name;
        self.save()
    }

    /// Get the container's name
    fn name(&mut self) -> Result<String, CatlibError> {
        self.sync()?;
        Ok(self.container_data.name.clone())
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn delete(&mut self) -> Result<(), CatlibError> {
        Model::delete(self)?;
        Ok(())
    }

    // Retrieve Container's uuid
    fn uuid(&self) -> Uuid {
        self.container_data.uuid
    }

    fn get_paths(&mut self) -> Result<Vec<ContainerPath>, CatlibError> {
        self.sync()?;
        Ok(self.container_data.paths.iter().cloned().collect())
    }
}

impl Model for Container {
    fn save(&self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("container-{}", self.container_data.uuid),
            ron::to_string(&self.container_data).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(
            self.db.clone(),
            format!("container-{}", self.container_data.uuid),
        )
    }

    fn sync(&mut self) -> CatlibResult<()> {
        self.container_data =
            fetch_container_data_by_uuid(self.db.clone(), &self.container_data.uuid)?;
        self.storages =
            fetch_storages_by_container_uuid(self.db.clone(), &self.container_data.uuid)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        sync::{Arc, Mutex},
    };

    use rstest::*;
    use wildland_corex::{entities::ContainerManifest, StorageTemplate};

    use super::db::test::catlib;
    use crate::*;

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

    fn make_container(catlib: &CatLib) -> Arc<Mutex<dyn ContainerManifest>> {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();
        let storage_template = StorageTemplate::try_new(
            "FoundationStorage",
            HashMap::from([
                (
                    "field1".to_owned(),
                    "Some value with container name: {{ CONTAINER_NAME }}".to_owned(),
                ),
                (
                    "parameter in key: {{ OWNER }}".to_owned(),
                    "enum: {{ ACCESS_MODE }}".to_owned(),
                ),
                ("uuid".to_owned(), "{{ CONTAINER_UUID }}".to_owned()),
                ("paths".to_owned(), "{{ PATHS }}".to_owned()),
            ]),
        )
        .unwrap();
        let locked_forest = forest.lock().unwrap();
        locked_forest
            .create_container("name".to_owned(), &storage_template)
            .unwrap()
    }

    #[rstest(catlib_with_forest as catlib)]
    fn fetch_created_container(catlib: CatLib) {
        let container = make_container(&catlib);
        let container = catlib
            .get_container(&container.lock().unwrap().uuid())
            .unwrap();

        assert_eq!(
            container
                .lock()
                .unwrap()
                .forest()
                .unwrap()
                .lock()
                .unwrap()
                .owner(),
            Identity([1; 32])
        );
    }

    #[rstest(catlib_with_forest as catlib)]
    fn fetch_created_container_from_forest_obj(catlib: CatLib) {
        let container = make_container(&catlib);
        let container = catlib
            .get_container(&container.lock().unwrap().uuid())
            .unwrap();

        assert_eq!(
            container
                .lock()
                .unwrap()
                .forest()
                .unwrap()
                .lock()
                .unwrap()
                .owner(),
            Identity([1; 32])
        );
    }

    #[rstest(catlib_with_forest as catlib)]
    fn container_with_paths(catlib: CatLib) {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        let container = make_container(&catlib);
        container
            .lock()
            .unwrap()
            .add_path("/foo/bar".to_string())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz".to_string())
            .unwrap();

        assert!(container
            .lock()
            .unwrap()
            .get_paths()
            .unwrap()
            .contains(&"/foo/bar".to_string()));
        assert!(container
            .lock()
            .unwrap()
            .get_paths()
            .unwrap()
            .contains(&"/bar/baz".to_string()));

        // Try to find that container in the database
        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/foo/bar".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);

        // Ensure again that it still has the paths
        assert!(container
            .lock()
            .unwrap()
            .get_paths()
            .unwrap()
            .contains(&"/foo/bar".to_string()));
        assert!(container
            .lock()
            .unwrap()
            .get_paths()
            .unwrap()
            .contains(&"/bar/baz".to_string()));

        // Try to fetch the same (one) container, using two different paths. The result
        // should be only one (not two) containers.
        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/foo/bar".to_string(), "/bar/baz".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);
    }

    #[rstest(catlib_with_forest as catlib)]
    fn multiple_containers_with_paths(catlib: CatLib) {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        let container = make_container(&catlib);
        container
            .lock()
            .unwrap()
            .add_path("/foo/bar".to_string())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz".to_string())
            .unwrap();

        // Create another container, that shares a path with the former
        let container = make_container(&catlib);
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz".to_string())
            .unwrap();

        // And yet another container that doesn't
        let container = make_container(&catlib);
        container
            .lock()
            .unwrap()
            .add_path("/what/ever".to_string())
            .unwrap();

        // try to find the first container
        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/foo/bar".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);

        // try to find the first and the second containers, using shared path
        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/bar/baz".to_string()], false)
            .unwrap();
        assert_eq!(containers.len(), 2);

        // Make sure that they are in fact two different containers
        assert_ne!(
            containers[0].lock().unwrap().uuid(),
            containers[1].lock().unwrap().uuid()
        );
    }

    // TODO (tkulik): Align UTs
    // #[rstest(catlib_with_forest as catlib)]
    // fn create_containers_with_different_storages(catlib: CatLib) {
    //     let alpha = make_container(&catlib);
    //     let beta = make_container(&catlib);

    //     let storage_template = StorageTemplate::try_new(
    //         "FoundationStorage",
    //         HashMap::from([
    //             (
    //                 "field1".to_owned(),
    //                 "Some value with container name: {{ CONTAINER_NAME }}".to_owned(),
    //             ),
    //             (
    //                 "parameter in key: {{ OWNER }}".to_owned(),
    //                 "enum: {{ ACCESS_MODE }}".to_owned(),
    //             ),
    //             ("uuid".to_owned(), "{{ CONTAINER_UUID }}".to_owned()),
    //             ("paths".to_owned(), "{{ PATHS }}".to_owned()),
    //         ]),
    //     )
    //     .unwrap();

    //     alpha
    //         .lock()
    //         .unwrap()
    //         .add_storage(&storage_template)
    //         .unwrap();
    //     alpha
    //         .lock()
    //         .unwrap()
    //         .add_storage(&storage_template)
    //         .unwrap();

    //     beta.lock().unwrap().add_storage(&storage_template).unwrap();

    //     let containers = catlib
    //         .find_containers_with_template(&Uuid::from_u128(2))
    //         .unwrap();

    //     assert_eq!(containers.len(), 1);
    //     assert_eq!(
    //         containers[0].lock().unwrap().uuid(),
    //         alpha.lock().unwrap().uuid()
    //     );

    //     let containers = catlib
    //         .find_containers_with_template(&Uuid::from_u128(1))
    //         .unwrap();

    //     assert_eq!(containers.len(), 2);
    //     assert_ne!(
    //         containers[0].lock().unwrap().uuid(),
    //         containers[1].lock().unwrap().uuid()
    //     );
    // }

    #[rstest(catlib_with_forest as catlib)]
    fn multiple_containers_with_subpaths(catlib: CatLib) {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        let container = make_container(&catlib);
        container
            .lock()
            .unwrap()
            .add_path("/foo/bar1".to_string())
            .unwrap();

        let container = make_container(&catlib);
        container
            .lock()
            .unwrap()
            .add_path("/foo/bar2".to_string())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz1".to_string())
            .unwrap();

        let container = make_container(&catlib);
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz2".to_string())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/baz/qux1".to_string())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/baz/qux2".to_string())
            .unwrap();

        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/foo".into()], false);
        assert_eq!(containers.err(), Some(CatlibError::NoRecordsFound));

        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/foo/bar1".into()], true)
            .unwrap();
        assert_eq!(containers.len(), 1);

        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/foo".into()], true)
            .unwrap();
        assert_eq!(containers.len(), 2);

        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/bar".into()], true)
            .unwrap();
        assert_eq!(containers.len(), 2);

        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/baz".into()], true)
            .unwrap();
        assert_eq!(containers.len(), 1);

        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/".into()], true)
            .unwrap();
        assert_eq!(containers.len(), 3);
    }
}
