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
use crate::{db::delete_model, db::save_model};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Create Forest object from its representation in Rust Object Notation
impl TryFrom<String> for Forest {
    type Error = ron::error::SpannedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ron::from_str(value.as_str())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Forest {
    uuid: Uuid,
    signers: Signers,
    owner: Identity,
    data: Vec<u8>,

    #[serde(skip, default = "use_default_database")]
    db: Rc<StoreDb>,
}

impl Forest {
    pub fn new(owner: Identity, signers: Signers, data: Vec<u8>, db: Rc<StoreDb>) -> Self {
        Forest {
            uuid: Uuid::new_v4(),
            signers,
            owner,
            data,
            db,
        }
    }
}

impl IForest for Forest {
    fn add_signer(&mut self, signer: Identity) -> CatlibResult<bool> {
        let added = self.signers.insert(signer);
        self.save()?;
        Ok(added)
    }

    fn del_signer(&mut self, signer: Identity) -> CatlibResult<bool> {
        let deleted = self.signers.remove(&signer);
        self.save()?;
        Ok(deleted)
    }

    fn containers(&self) -> CatlibResult<Vec<Container>> {
        self.db.load()?;
        let data = self.db.read(|db| db.clone()).map_err(CatlibError::from)?;

        let containers: Vec<Container> = data
            .iter()
            .filter(|(id, _)| (**id).starts_with("container-"))
            .map(|(_, container_str)| Container::try_from((*container_str).clone()).unwrap())
            .filter(|container| {
                container.forest().is_ok() && container.forest().unwrap().uuid() == self.uuid()
            })
            .collect();

        match containers.len() {
            0 => Err(CatlibError::NoRecordsFound),
            _ => Ok(containers),
        }
    }

    fn remove(&mut self) -> CatlibResult<bool> {
        self.delete()?;
        Ok(true)
    }

    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn owner(&self) -> Identity {
        self.owner.clone()
    }

    fn signers(&self) -> Signers {
        self.signers.clone()
    }

    fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn update(&mut self, data: Vec<u8>) -> CatlibResult<()> {
        self.data = data;
        self.save()
    }

    fn create_container(&self) -> CatlibResult<Container> {
        let mut container = Container::new(self.uuid(), self.db.clone());
        container.save()?;

        Ok(container)
    }

    fn create_bridge(&self, path: ContainerPath, link_data: Vec<u8>) -> CatlibResult<Bridge> {
        let mut bridge = Bridge::new(self.uuid(), path, link_data, self.db.clone());
        bridge.save()?;

        Ok(bridge)
    }

    fn find_bridge(&self, path: ContainerPath) -> CatlibResult<Bridge> {
        self.db.load()?;
        let data = self.db.read(|db| db.clone()).map_err(CatlibError::from)?;

        let bridges: Vec<Bridge> = data
            .iter()
            .filter(|(id, _)| (**id).starts_with("bridge-"))
            .map(|(_, bridge_str)| Bridge::try_from((*bridge_str).clone()).unwrap())
            .filter(|bridge| {
                bridge.forest().unwrap().uuid() == self.uuid() && bridge.path() == path
            })
            .collect();

        match bridges.len() {
            0 => Err(CatlibError::NoRecordsFound),
            1 => Ok(bridges[0].clone()),
            _ => Err(CatlibError::MalformedDatabaseEntry),
        }
    }

    fn find_containers(
        &self,
        paths: Vec<String>,
        include_subdirs: bool,
    ) -> CatlibResult<Vec<Container>> {
        self.db.load()?;
        let data = self.db.read(|db| db.clone()).map_err(CatlibError::from)?;

        let containers: Vec<Container> = data
            .iter()
            .filter(|(id, _)| (**id).starts_with("container-"))
            .map(|(_, container_str)| Container::try_from((*container_str).clone()).unwrap())
            .filter(|container| {
                container.forest().unwrap().uuid() == self.uuid()
                    && container.paths().iter().any(|container_path| {
                        paths.iter().any(|path| {
                            (include_subdirs && container_path.starts_with(path))
                                || container_path.eq(path)
                        })
                    })
            })
            .collect();

        match containers.len() {
            0 => Err(CatlibError::NoRecordsFound),
            _ => Ok(containers),
        }
    }
}

impl Model for Forest {
    fn save(&mut self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("forest-{}", self.uuid()),
            ron::to_string(self).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(self.db.clone(), format!("forest-{}", self.uuid()))
    }
}

#[cfg(test)]
mod tests {
    use super::Forest;
    use crate::*;
    use rstest::*;
    use uuid::Bytes;

    #[fixture]
    fn catlib() -> CatLib {
        db::init_catlib(rand::random::<Bytes>())
    }

    fn make_forest(catlib: &CatLib) -> Forest {
        let owner = Identity([1; 32]);

        catlib.create_forest(owner, Signers::new(), vec![]).unwrap()
    }

    fn make_forest_with_signer(catlib: &CatLib) -> Forest {
        let owner = Identity([1; 32]);
        let signer = Identity([2; 32]);

        let mut signers = Signers::new();
        signers.insert(signer);

        catlib.create_forest(owner, signers, vec![]).unwrap()
    }

    #[rstest]
    fn read_new_forest(catlib: CatLib) {
        make_forest_with_signer(&catlib);

        let forest = catlib.find_forest(Identity([1; 32])).unwrap();

        assert_eq!(forest.owner, Identity([1; 32]));
        assert_eq!(forest.signers.len(), 1);
    }

    #[rstest]
    fn read_new_forest_by_uuid(catlib: CatLib) {
        let f = make_forest_with_signer(&catlib);

        let forest = catlib.get_forest(f.uuid()).unwrap();

        assert_eq!(forest.owner, Identity([1; 32]));
        assert_eq!(forest.signers.len(), 1);
    }

    #[rstest]
    fn create_two_different_forests(catlib: CatLib) {
        make_forest(&catlib);
        catlib
            .create_forest(Identity([2; 32]), Signers::new(), vec![])
            .unwrap();

        let forest = catlib.find_forest(Identity([1; 32])).unwrap();

        assert_eq!(forest.owner(), Identity([1; 32]));

        let forest = catlib.find_forest(Identity([2; 32])).unwrap();

        assert_eq!(forest.owner(), Identity([2; 32]));
    }

    #[rstest]
    fn read_non_existing_forest(catlib: CatLib) {
        let forest = catlib.find_forest(Identity([1; 32]));

        assert_eq!(forest.err(), Some(CatlibError::NoRecordsFound));
    }

    #[rstest]
    fn read_wrong_forest_owner(catlib: CatLib) {
        make_forest(&catlib);

        let forest = catlib.find_forest(Identity([0; 32]));

        assert_eq!(forest.err(), Some(CatlibError::NoRecordsFound));
    }

    #[rstest]
    fn add_forest_data(catlib: CatLib) {
        let mut f = make_forest(&catlib);

        f.update(b"some data".to_vec()).unwrap();

        let forest = catlib.find_forest(f.owner()).unwrap();

        assert_eq!(forest.data(), b"some data".to_vec());
    }

    #[rstest]
    fn delete_empty_forest(catlib: CatLib) {
        let mut f = make_forest(&catlib);

        f.delete().unwrap();

        assert!(matches!(
            catlib.find_forest(f.owner()),
            Err(CatlibError::NoRecordsFound)
        ));
    }

    #[rstest]
    fn delete_forest_with_data(catlib: CatLib) {
        let mut f = make_forest_with_signer(&catlib);

        f.delete().unwrap();

        assert!(matches!(
            catlib.find_forest(f.owner()),
            Err(CatlibError::NoRecordsFound)
        ));
    }

    #[rstest]
    fn add_forest_data_and_fetch_twice(catlib: CatLib) {
        let mut f = make_forest(&catlib);

        f.update(b"some data".to_vec()).unwrap();

        let mut forest = catlib.find_forest(f.owner()).unwrap();

        assert_eq!(forest.data(), b"some data".to_vec());

        forest.update(b"updated data".to_vec()).unwrap();

        let forest = catlib.find_forest(f.owner()).unwrap();

        assert_eq!(forest.data(), b"updated data".to_vec());
    }

    #[rstest]
    fn adding_signers(catlib: CatLib) {
        let alice = Identity([3; 32]);
        let bob = Identity([4; 32]);
        let charlie = Identity([5; 32]);

        let mut forest = make_forest_with_signer(&catlib);

        assert_eq!(forest.owner, Identity([1; 32]));

        assert_eq!(forest.signers.len(), 1);

        forest.add_signer(alice).unwrap();

        // Find the same forest by it's owner and add one more signer
        let mut forest = catlib.find_forest(Identity([1; 32])).unwrap();
        forest.add_signer(bob).unwrap();
        assert_eq!(forest.signers.len(), 3);

        // Add one more...
        forest.add_signer(charlie).unwrap();

        // ...but this trime fetch with uuid
        let forest = catlib.get_forest(forest.uuid()).unwrap();
        assert_eq!(forest.signers.len(), 4);
    }
}
