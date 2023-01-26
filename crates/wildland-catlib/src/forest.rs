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

use std::path::PathBuf;

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use wildland_corex::catlib_service::entities::{
    BridgeManifest,
    ContainerManifest,
    ForestManifest,
    Identity,
    Signers,
};
use wildland_corex::{ContainerPath, ContainerPaths};

use super::*;
use crate::container::ContainerData;

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

impl From<&ForestEntity> for String {
    fn from(value: &ForestEntity) -> Self {
        ron::to_string(&value.data).unwrap()
    }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) struct ForestEntity {
    pub(crate) data: ForestData,

    #[derivative(Debug = "ignore")]
    pub(crate) db: RedisDb,
}

impl ForestEntity {
    pub fn new(
        owner: Identity,
        signers: Signers,
        data: Vec<u8>,
        db: RedisDb,
    ) -> Result<Self, CatlibError> {
        let forest = Self {
            data: ForestData {
                uuid: Uuid::new_v4(),
                signers,
                owner,
                data,
            },
            db,
        };
        forest.save()?;
        Ok(forest)
    }

    pub fn from_forest_data(forest_data: ForestData, db: RedisDb) -> Self {
        Self {
            data: forest_data,
            db,
        }
    }
}

impl ForestManifest for ForestEntity {
    fn add_signer(&mut self, signer: Identity) -> CatlibResult<bool> {
        let added = self.data.signers.insert(signer);
        self.save()?;
        Ok(added)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn delete_signer(&mut self, signer: Identity) -> CatlibResult<bool> {
        let deleted = self.data.signers.remove(&signer);
        self.save()?;
        Ok(deleted)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if Forest has no [`Container`].
    /// - Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn containers(&self) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>> {
        db::fetch_containers_by_forest_uuid(self.db.clone(), &self.data.uuid)
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<()> {
        self.data.data = data;
        self.save()?;
        Ok(())
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn remove(&mut self) -> CatlibResult<bool> {
        Model::delete(self)?;
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn create_container(
        &self,
        container_uuid: Uuid,
        forest_uuid: Uuid,
        name: String,
        path: ContainerPath,
    ) -> Result<Arc<Mutex<dyn ContainerManifest>>, CatlibError> {
        let container_data = ContainerData {
            uuid: container_uuid,
            forest_uuid,
            name,
            paths: ContainerPaths::from([path]),
        };
        let container = ContainerEntity::from_container_data(container_data, self.db.clone());
        container.save()?;
        Ok(Arc::new(Mutex::new(container)))
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
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
        path: String,
        link_data: Vec<u8>,
    ) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError> {
        let bridge = Bridge::new(
            self.data.uuid,
            PathBuf::from(path),
            link_data,
            self.db.clone(),
        );
        bridge.save()?;
        Ok(Arc::new(Mutex::new(bridge)))
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Bridge`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Bridge`] was found.
    /// - Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_bridge(&self, path: String) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError> {
        db::fetch_bridge_by_path(self.db.clone(), &self.uuid(), path)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    /// - Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
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
    /// container.lock().unwrap().add_path("/foo/bar".into());
    ///
    /// let containers = forest.find_containers(vec!["/foo/bar".to_string()], false).unwrap();
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_containers(
        &self,
        paths: ContainerPaths,
        include_subdirs: bool,
    ) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>> {
        db::fetch_containers_by_path(self.db.clone(), &self.uuid(), paths, include_subdirs)
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

    fn serialise(&self) -> String {
        self.into()
    }
}

impl Model for ForestEntity {
    fn save(&self) -> CatlibResult<()> {
        db::commands::set(
            self.db.clone(),
            format!("forest-{}", self.uuid()),
            self.serialise(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        db::commands::delete(self.db.clone(), format!("forest-{}", self.uuid()))
    }

    fn sync(&mut self) -> CatlibResult<()> {
        let forest = db::fetch_forest_by_uuid(self.db.clone(), &self.uuid())?;
        let forest_lock = forest.lock().expect("Poisoned Mutex");
        self.data = ForestData::from(forest_lock.serialise().as_str());
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
