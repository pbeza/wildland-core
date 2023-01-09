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

use std::rc::Rc;

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use wildland_corex::catlib_service::entities::{
    BridgeManifest,
    ContainerManifest,
    ContainerPath,
    ForestManifest,
    Identity,
    Signers,
};
use wildland_corex::StorageTemplate;

use super::*;
use crate::bridge::BridgeData;
use crate::container::ContainerData;
use crate::db::{delete_model, save_model};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ForestData {
    pub uuid: Uuid,
    pub signers: Signers,
    pub owner: Identity,
    pub data: Vec<u8>,
}

impl From<&str> for ForestData {
    fn from(str_data: &str) -> Self {
        ron::from_str(str_data).unwrap()
    }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) struct Forest {
    pub(crate) data: ForestData,

    #[derivative(Debug = "ignore")]
    pub(crate) db: Rc<StoreDb>,
}

impl Forest {
    pub fn new(owner: Identity, signers: Signers, data: Vec<u8>, db: Rc<StoreDb>) -> Self {
        Self {
            data: ForestData {
                uuid: Uuid::new_v4(),
                signers,
                owner,
                data,
            },
            db,
        }
    }
}

impl ForestManifest for Forest {
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn add_signer(&mut self, signer: Identity) -> CatlibResult<bool> {
        let added = self.data.signers.insert(signer);
        self.save()?;
        Ok(added)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn del_signer(&mut self, signer: Identity) -> CatlibResult<bool> {
        let deleted = self.data.signers.remove(&signer);
        self.save()?;
        Ok(deleted)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if Forest has no [`Container`].
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn containers(&self) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>> {
        self.db.load().map_err(to_catlib_error)?;
        let data = self.db.read(|db| db.clone()).map_err(to_catlib_error)?;
        let containers: Result<Vec<Arc<Mutex<dyn ContainerManifest>>>, CatlibError> = data
            .iter()
            .filter(|(id, _)| id.starts_with("container-"))
            .map(|(_, container_str)| ContainerData::from(container_str.as_str()))
            .filter(|container_data| container_data.forest_uuid == self.data.uuid)
            .map(|container_data| {
                Container::from_container_data(container_data, self.db.clone()).map(|container| {
                    Arc::new(Mutex::new(container)) as Arc<Mutex<dyn ContainerManifest>>
                })
            })
            .collect();
        let containers = containers?;

        match containers.len() {
            0 => Err(CatlibError::NoRecordsFound),
            _ => Ok(containers),
        }
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<()> {
        self.data.data = data;
        self.save()?;
        Ok(())
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn remove(&mut self) -> CatlibResult<bool> {
        Model::delete(self)?;
        Ok(true)
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
    /// # use wildland_corex::StorageTemplate;
    /// # use wildland_corex::interface::CatLib as ICatLib;
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
    /// let path = "/some/path".to_owned();
    /// let container = forest.lock().unwrap().create_container("container name1".to_owned(), &storage_template, path).unwrap();
    /// container.lock().unwrap().add_path("/foo/bar1".to_string());
    /// container.lock().unwrap().add_path("/bar/baz1".to_string());
    /// ```
    #[tracing::instrument(level = "debug", skip_all)]
    fn create_container(
        &self,
        name: String,
        storage_template: &StorageTemplate,
        path: ContainerPath,
    ) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>> {
        let forest_owner = Arc::new(Mutex::new(self.clone()));
        Ok(Arc::new(Mutex::new(Container::new(
            forest_owner,
            storage_template,
            name,
            path,
            self.db.clone(),
        )?)))
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use wildland_corex::catlib_service::entities::Identity;
    /// # use wildland_corex::catlib_service::interface::CatLib as ICatLib;
    /// # use std::collections::HashSet;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// forest.lock().unwrap().create_bridge("/other/forest".to_string(), vec![]);
    /// ```
    #[tracing::instrument(level = "debug", skip_all)]
    fn create_bridge(
        &self,
        path: ContainerPath,
        link_data: Vec<u8>,
    ) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError> {
        let bridge = Bridge::new(self.data.uuid, path, link_data, self.db.clone());
        bridge.save()?;

        Ok(Arc::new(Mutex::new(bridge)))
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Bridge`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Bridge`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_bridge(
        &self,
        path: ContainerPath,
    ) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError> {
        self.db.load().map_err(to_catlib_error)?;
        let data = self.db.read(|db| db.clone()).map_err(to_catlib_error)?;

        let bridges: Vec<_> = data
            .iter()
            .filter(|(id, _)| id.starts_with("bridge-"))
            .map(|(_, bridge_str)| Bridge {
                data: BridgeData::from(bridge_str.as_str()),
                db: self.db.clone(),
            })
            .filter(|bridge| bridge.data.forest_uuid == self.data.uuid && bridge.data.path == path)
            .map(|bridge| Arc::new(Mutex::new(bridge)))
            .collect();

        match bridges.len() {
            0 => Err(CatlibError::NoRecordsFound),
            1 => Ok(bridges[0].clone()),
            _ => Err(CatlibError::MalformedDatabaseRecord),
        }
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use wildland_corex::catlib_service::entities::Identity;
    /// # use wildland_corex::catlib_service::interface::CatLib as ICatLib;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  b"owner".to_vec(),
    ///                  HashSet::from([b"signer".to_vec()]),
    ///                  vec![],
    ///              ).unwrap();
    /// let container = forest.lock().unwrap().create_container("container name".to_owned()).unwrap();
    /// container.lock().unwrap().add_path("/foo/bar".to_string());
    ///
    /// let containers = forest.find_containers(vec!["/foo/bar".to_string()], false).unwrap();
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_containers(
        &self,
        paths: Vec<String>,
        include_subdirs: bool,
    ) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>> {
        self.db.load().map_err(to_catlib_error)?;
        let data = self.db.read(|db| db.clone()).map_err(to_catlib_error)?;

        let containers: Result<Vec<Arc<Mutex<dyn ContainerManifest>>>, CatlibError> = data
            .iter()
            .filter(|(id, _)| id.starts_with("container-"))
            .map(|(_, container_str)| ContainerData::from(container_str.as_str()))
            .filter(|container_data| container_data.forest_uuid == self.data.uuid)
            .filter(|container_data| {
                container_data.paths.iter().any(|container_path| {
                    paths.iter().any(|path| {
                        (include_subdirs && container_path.starts_with(path))
                            || container_path.eq(path)
                    })
                })
            })
            .map(|container_data| {
                Container::from_container_data(container_data, self.db.clone()).map(|container| {
                    Arc::new(Mutex::new(container)) as Arc<Mutex<dyn ContainerManifest>>
                })
            })
            .collect();
        let containers = containers?;

        match containers.len() {
            0 => Err(CatlibError::NoRecordsFound),
            _ => Ok(containers),
        }
    }

    fn data(&mut self) -> CatlibResult<Vec<u8>> {
        self.sync()?;
        Ok(self.data.data.clone())
    }

    fn uuid(&self) -> Uuid {
        self.data.uuid // Uuid should not change - no need to sync with db
    }

    fn owner(&self) -> Identity {
        self.data.owner.clone()
    }

    fn signers(&mut self) -> CatlibResult<Signers> {
        self.sync()?;
        Ok(self.data.signers.clone())
    }
}

impl Model for Forest {
    fn save(&self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("forest-{}", self.data.uuid),
            ron::to_string(&self.data).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(self.db.clone(), format!("forest-{}", self.data.uuid))
    }

    fn sync(&mut self) -> CatlibResult<()> {
        let forest_data = fetch_forest_data_by_uuid(self.db.clone(), &self.data.uuid)?;
        self.data = forest_data;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use wildland_corex::catlib_service::entities::ForestManifest;

    use super::db::test::catlib;
    use crate::*;

    fn make_forest(catlib: &CatLib) -> Arc<Mutex<dyn ForestManifest>> {
        let owner = Identity([1; 32]);

        catlib.create_forest(owner, Signers::new(), vec![]).unwrap()
    }

    fn make_forest_with_signer(catlib: &CatLib) -> Arc<Mutex<dyn ForestManifest>> {
        let owner = Identity([1; 32]);
        let signer = Identity([2; 32]);

        let mut signers = Signers::new();
        signers.insert(signer);

        catlib.create_forest(owner, signers, vec![]).unwrap()
    }

    #[rstest]
    fn read_new_forest(catlib: CatLib) {
        make_forest_with_signer(&catlib);

        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        assert_eq!(forest.lock().unwrap().owner(), Identity([1; 32]));
        assert_eq!(forest.lock().unwrap().signers().unwrap().len(), 1);
    }

    #[rstest]
    fn read_new_forest_by_uuid(catlib: CatLib) {
        let f = make_forest_with_signer(&catlib);

        let forest = catlib.get_forest(&f.lock().unwrap().uuid()).unwrap();

        assert_eq!(forest.lock().unwrap().owner(), Identity([1; 32]));
        assert_eq!(forest.lock().unwrap().signers().unwrap().len(), 1);
    }

    #[rstest]
    fn create_two_different_forests(catlib: CatLib) {
        make_forest(&catlib);
        catlib
            .create_forest(Identity([2; 32]), Signers::new(), vec![])
            .unwrap();

        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        assert_eq!((*forest).lock().unwrap().owner(), Identity([1; 32]));

        let forest = catlib.find_forest(&Identity([2; 32])).unwrap();

        assert_eq!((*forest).lock().unwrap().owner(), Identity([2; 32]));
    }

    #[rstest]
    fn read_non_existing_forest(catlib: CatLib) {
        let forest = catlib.find_forest(&Identity([1; 32]));

        assert_eq!(forest.err(), Some(CatlibError::NoRecordsFound));
    }

    #[rstest]
    fn read_wrong_forest_owner(catlib: CatLib) {
        make_forest(&catlib);

        let forest = catlib.find_forest(&Identity([0; 32]));

        assert_eq!(forest.err(), Some(CatlibError::NoRecordsFound));
    }

    #[rstest]
    fn add_forest_data(catlib: CatLib) {
        let f = make_forest(&catlib);

        f.lock().unwrap().update(b"some data".to_vec()).unwrap();

        let forest = catlib.find_forest(&f.lock().unwrap().owner()).unwrap();

        assert_eq!(
            forest.lock().unwrap().data().unwrap(),
            b"some data".to_vec()
        );
    }

    #[rstest]
    fn delete_empty_forest(catlib: CatLib) {
        let f = make_forest(&catlib);

        f.lock().unwrap().remove().unwrap();

        assert!(matches!(
            catlib.find_forest(&f.lock().unwrap().owner()),
            Err(CatlibError::NoRecordsFound)
        ));
    }

    #[rstest]
    fn delete_forest_with_data(catlib: CatLib) {
        let f = make_forest_with_signer(&catlib);

        f.lock().unwrap().remove().unwrap();

        assert!(matches!(
            catlib.find_forest(&f.lock().unwrap().owner()),
            Err(CatlibError::NoRecordsFound)
        ));
    }

    #[rstest]
    fn add_forest_data_and_fetch_twice(catlib: CatLib) {
        let f = make_forest(&catlib);

        f.lock().unwrap().update(b"some data".to_vec()).unwrap();

        let forest = catlib.find_forest(&f.lock().unwrap().owner()).unwrap();

        assert_eq!(
            forest.lock().unwrap().data().unwrap(),
            b"some data".to_vec()
        );

        forest
            .lock()
            .unwrap()
            .update(b"updated data".to_vec())
            .unwrap();

        let forest = catlib.find_forest(&f.lock().unwrap().owner()).unwrap();

        assert_eq!(
            forest.lock().unwrap().data().unwrap(),
            b"updated data".to_vec()
        );
    }

    #[rstest]
    fn adding_signers(catlib: CatLib) {
        let alice = Identity([3; 32]);
        let bob = Identity([4; 32]);
        let charlie = Identity([5; 32]);

        let forest = make_forest_with_signer(&catlib);

        assert_eq!(forest.lock().unwrap().owner(), Identity([1; 32]));

        assert_eq!(forest.lock().unwrap().signers().unwrap().len(), 1);

        forest.lock().unwrap().add_signer(alice).unwrap();

        // Find the same forest by it's owner and add one more signer
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();
        forest.lock().unwrap().add_signer(bob).unwrap();
        assert_eq!(forest.lock().unwrap().signers().unwrap().len(), 3);

        // Add one more...
        forest.lock().unwrap().add_signer(charlie).unwrap();

        // ...but this trime fetch with uuid
        let forest = catlib.get_forest(&forest.lock().unwrap().uuid()).unwrap();
        assert_eq!(forest.lock().unwrap().signers().unwrap().len(), 4);
    }
}
