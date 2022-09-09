use super::*;
use crate::{db::delete_model, db::save_model, error::*};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

impl TryFrom<String> for Forest {
    type Error = ron::error::SpannedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ron::from_str(value.as_str())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Forest {
    uuid: String,
    signers: Signers,
    owner: Identity,
    data: Vec<u8>,

    #[serde(skip, default = "use_default_database")]
    db: Rc<StoreDb>,
}

impl Forest {
    pub fn new(owner: Identity, signers: Signers, data: Vec<u8>, db: Rc<StoreDb>) -> Self {
        Forest {
            uuid: Uuid::new_v4().to_string(), // redundant?
            signers,
            owner,
            data,
            db,
        }
    }
}

impl crate::contracts::Forest for Forest {
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

    fn remove(&mut self) -> CatlibResult<bool> {
        self.delete()?;
        Ok(true)
    }

    fn uuid(&self) -> String {
        self.uuid.clone()
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
        let owner = b"owner".to_vec();

        catlib.create_forest(owner, Signers::new(), vec![]).unwrap()
    }

    fn make_forest_with_signer(catlib: &CatLib) -> Forest {
        let owner = b"owner".to_vec();
        let signer = b"signer".to_vec();

        let mut signers = Signers::new();
        signers.insert(signer);

        catlib.create_forest(owner, signers, vec![]).unwrap()
    }

    #[rstest]
    fn read_new_forest(catlib: CatLib) {
        make_forest_with_signer(&catlib);

        let forest = catlib.find_forest(b"owner".to_vec()).unwrap();

        assert_eq!(forest.owner, b"owner");
        assert_eq!(forest.signers.len(), 1);
    }

    #[rstest]
    fn read_new_forest_by_uuid(catlib: CatLib) {
        let f = make_forest_with_signer(&catlib);

        let forest = catlib.get_forest(f.uuid()).unwrap();

        assert_eq!(forest.owner, b"owner");
        assert_eq!(forest.signers.len(), 1);
    }

    #[rstest]
    fn create_two_different_forests(catlib: CatLib) {
        make_forest(&catlib);
        catlib
            .create_forest(b"another owner".to_vec(), Signers::new(), vec![])
            .unwrap();

        let forest = catlib.find_forest(b"owner".to_vec()).unwrap();

        assert_eq!(forest.owner(), b"owner");

        let forest = catlib.find_forest(b"another owner".to_vec()).unwrap();

        assert_eq!(forest.owner(), b"another owner");
    }

    #[rstest]
    fn read_non_existing_forest(catlib: CatLib) {
        let forest = catlib.find_forest(b"owner".to_vec());

        assert_eq!(forest.err(), Some(CatlibError::NoRecordsFound));
    }

    #[rstest]
    fn read_wrong_forest_owner(catlib: CatLib) {
        make_forest(&catlib);

        let forest = catlib.find_forest(b"non_existing_owner".to_vec());

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
        let alice = b"alice".to_vec();
        let bob = b"bob".to_vec();
        let charlie = b"charlie".to_vec();

        let mut forest = make_forest_with_signer(&catlib);

        assert_eq!(forest.owner, b"owner");

        assert_eq!(forest.signers.len(), 1);

        forest.add_signer(alice).unwrap();

        // Find the same forest by it's owner and add one more signer
        let mut forest = catlib.find_forest(b"owner".to_vec()).unwrap();
        forest.add_signer(bob).unwrap();
        assert_eq!(forest.signers.len(), 3);

        // Add one more...
        forest.add_signer(charlie).unwrap();

        // ...but this trime fetch with uuid
        let forest = catlib.get_forest(forest.uuid()).unwrap();
        assert_eq!(forest.signers.len(), 4);
    }
}
