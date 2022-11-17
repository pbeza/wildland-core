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

use wildland_corex::entities::{
    Bridge as IBridge, Container as IContainer, ContainerPath, Forest as IForest, ForestData,
    Identity, Signers,
};

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct Forest {
    data: ForestData,
}

impl Forest {
    pub fn new(owner: Identity, signers: Signers, data: Vec<u8>) -> Self {
        Self {
            data: ForestData {
                uuid: Uuid::new_v4(),
                signers,
                owner,
                data,
            },
        }
    }

    pub fn from_db_entry(value: &str) -> Self {
        let data = serde_yaml::from_str(value).unwrap();
        Self { data }
    }
}

impl AsRef<ForestData> for Forest {
    fn as_ref(&self) -> &ForestData {
        &self.data
    }
}

impl IForest for Forest {
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
    fn del_signer(&mut self, signer: Identity) -> CatlibResult<bool> {
        let deleted = self.data.signers.remove(&signer);
        self.save()?;
        Ok(deleted)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if Forest has no [`Container`].
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn containers(&self) -> CatlibResult<Vec<Box<dyn IContainer>>> {
        todo!()
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<&mut dyn IForest> {
        self.data.data = data;
        self.save()?;
        Ok(self)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn delete(&mut self) -> CatlibResult<bool> {
        Model::delete(self)?;
        Ok(true)
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
    /// container.add_path("/foo/bar".to_string());
    /// container.add_path("/bar/baz".to_string());
    /// ```
    fn create_container(&self, name: String) -> CatlibResult<Box<dyn IContainer>> {
        let mut container = Box::new(Container::new(self.data.uuid, name));
        container.save()?;

        Ok(container)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::interface::CatLib as ICatLib;
    /// # use std::collections::HashSet;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// forest.create_bridge("/other/forest".to_string(), vec![]);
    /// ```
    fn create_bridge(
        &self,
        path: ContainerPath,
        link_data: Vec<u8>,
    ) -> CatlibResult<Box<dyn IBridge>> {
        let mut bridge = Box::new(Bridge::new(self.data.uuid, path, link_data));
        bridge.save()?;

        Ok(bridge)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Bridge`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Bridge`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn find_bridge(&self, _path: ContainerPath) -> CatlibResult<Box<dyn IBridge>> {
        todo!()
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::interface::CatLib as ICatLib;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  b"owner".to_vec(),
    ///                  HashSet::from([b"signer".to_vec()]),
    ///                  vec![],
    ///              ).unwrap();
    /// let mut container = forest.create_container("container name".to_owned()).unwrap();
    /// container.add_path("/foo/bar".to_string());
    ///
    /// let containers = forest.find_containers(vec!["/foo/bar".to_string()], false).unwrap();
    fn find_containers(
        &self,
        _paths: Vec<String>,
        _include_subdirs: bool,
    ) -> CatlibResult<Vec<Box<dyn IContainer>>> {
        todo!()
    }
}

impl Model for Forest {
    fn save(&mut self) -> CatlibResult<()> {
        todo!()
    }

    fn delete(&mut self) -> CatlibResult<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::db::test::catlib;
    use crate::*;
    use rstest::*;
    use wildland_corex::catlib_service::entities::Forest;

    fn make_forest(catlib: &CatLib) -> Box<dyn Forest> {
        let owner = Identity([1; 32]);

        catlib.create_forest(owner, Signers::new(), vec![]).unwrap()
    }

    fn make_forest_with_signer(catlib: &CatLib) -> Box<dyn Forest> {
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

        assert_eq!((*forest).as_ref().owner, Identity([1; 32]));
        assert_eq!((*forest).as_ref().signers.len(), 1);
    }

    #[rstest]
    fn read_new_forest_by_uuid(catlib: CatLib) {
        let f = make_forest_with_signer(&catlib);

        let forest = catlib.get_forest(&(*f).as_ref().uuid).unwrap();

        assert_eq!((*forest).as_ref().owner, Identity([1; 32]));
        assert_eq!((*forest).as_ref().signers.len(), 1);
    }

    #[rstest]
    fn create_two_different_forests(catlib: CatLib) {
        make_forest(&catlib);
        catlib
            .create_forest(Identity([2; 32]), Signers::new(), vec![])
            .unwrap();

        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        assert_eq!((*forest).as_ref().owner, Identity([1; 32]));

        let forest = catlib.find_forest(&Identity([2; 32])).unwrap();

        assert_eq!((*forest).as_ref().owner, Identity([2; 32]));
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
        let mut f = make_forest(&catlib);

        f.update(b"some data".to_vec()).unwrap();

        let forest = catlib.find_forest(&(*f).as_ref().owner).unwrap();

        assert_eq!((*forest).as_ref().data, b"some data".to_vec());
    }

    #[rstest]
    fn delete_empty_forest(catlib: CatLib) {
        let mut f = make_forest(&catlib);

        f.delete().unwrap();

        assert!(matches!(
            catlib.find_forest(&(*f).as_ref().owner),
            Err(CatlibError::NoRecordsFound)
        ));
    }

    #[rstest]
    fn delete_forest_with_data(catlib: CatLib) {
        let mut f = make_forest_with_signer(&catlib);

        f.delete().unwrap();

        assert!(matches!(
            catlib.find_forest(&(*f).as_ref().owner),
            Err(CatlibError::NoRecordsFound)
        ));
    }

    #[rstest]
    fn add_forest_data_and_fetch_twice(catlib: CatLib) {
        let mut f = make_forest(&catlib);

        f.update(b"some data".to_vec()).unwrap();

        let mut forest = catlib.find_forest(&(*f).as_ref().owner).unwrap();

        assert_eq!((*forest).as_ref().data, b"some data".to_vec());

        forest.update(b"updated data".to_vec()).unwrap();

        let forest = catlib.find_forest(&(*f).as_ref().owner).unwrap();

        assert_eq!((*forest).as_ref().data, b"updated data".to_vec());
    }

    #[rstest]
    fn adding_signers(catlib: CatLib) {
        let alice = Identity([3; 32]);
        let bob = Identity([4; 32]);
        let charlie = Identity([5; 32]);

        let mut forest = make_forest_with_signer(&catlib);

        assert_eq!((*forest).as_ref().owner, Identity([1; 32]));

        assert_eq!((*forest).as_ref().signers.len(), 1);

        forest.add_signer(alice).unwrap();

        // Find the same forest by it's owner and add one more signer
        let mut forest = catlib.find_forest(&Identity([1; 32])).unwrap();
        forest.add_signer(bob).unwrap();
        assert_eq!((*forest).as_ref().signers.len(), 3);

        // Add one more...
        forest.add_signer(charlie).unwrap();

        // ...but this trime fetch with uuid
        let forest = catlib.get_forest(&(*forest).as_ref().uuid).unwrap();
        assert_eq!((*forest).as_ref().signers.len(), 4);
    }
}
