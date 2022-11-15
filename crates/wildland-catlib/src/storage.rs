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
use std::{rc::Rc, str::FromStr};
use wildland_corex::entities::{Container, Storage as IStorage, StorageData};

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct Storage {
    data: StorageData,

    #[derivative(Debug = "ignore")]
    db: Rc<StoreDb>,
}

/// Create Storage object from its representation in Rust Object Notation
impl FromStr for Storage {
    type Err = ron::error::SpannedError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let data = ron::from_str(value)?;
        Ok(Self::from_data_and_db(data, use_default_database()))
    }
}

impl Storage {
    pub fn new(
        container_uuid: Uuid,
        template_uuid: Option<Uuid>,
        data: Vec<u8>,
        db: Rc<StoreDb>,
    ) -> Self {
        Self {
            data: StorageData {
                uuid: Uuid::new_v4(),
                container_uuid,
                template_uuid,
                data,
            },
            db,
        }
    }

    pub fn from_data_and_db(data: StorageData, db: Rc<StoreDb>) -> Self {
        Self { data, db }
    }
}

impl AsRef<StorageData> for Storage {
    fn as_ref(&self) -> &StorageData {
        &self.data
    }
}

impl IStorage for Storage {
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Container`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn container(&self) -> CatlibResult<Box<dyn Container>> {
        fetch_container_by_uuid(self.db.clone(), &self.data.container_uuid)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<&mut dyn IStorage> {
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
}

impl Model for Storage {
    fn save(&mut self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("storage-{}", self.data.uuid),
            ron::to_string(&self.data).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(self.db.clone(), format!("storage-{}", self.data.uuid))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::*;
    use rstest::*;
    use uuid::Bytes;
    use wildland_corex::catlib_service::entities::{Container, Storage};

    fn _catlib() -> CatLib {
        let catlib = db::init_catlib(rand::random::<Bytes>());

        // Create a dummy forest and container to which storages will be bound
        catlib
            .create_forest(
                Identity([1; 32]),
                HashSet::from([Identity([2; 32])]),
                vec![],
            )
            .unwrap();

        catlib
    }

    fn _container(catlib: &CatLib) -> Box<dyn Container> {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();
        forest.create_container("name".to_owned()).unwrap()
    }

    #[fixture]
    fn catlib() -> CatLib {
        _catlib()
    }

    #[fixture]
    fn container() -> Box<dyn Container> {
        let catlib = _catlib();
        _container(&catlib)
    }

    fn make_storage(container: &dyn Container) -> Box<dyn Storage> {
        container.create_storage(None, vec![]).unwrap()
    }

    fn make_storage_with_template(
        container: &dyn Container,
        template_id: Uuid,
    ) -> Box<dyn Storage> {
        container.create_storage(Some(template_id), vec![]).unwrap()
    }

    #[rstest]
    fn create_empty_storage(container: Box<dyn Container>) {
        make_storage(container.as_ref());

        assert_eq!(container.storages().unwrap().len(), 1);

        make_storage(container.as_ref());

        assert_eq!(container.storages().unwrap().len(), 2);
    }

    #[rstest]
    fn delete_a_storage(container: Box<dyn Container>) {
        make_storage(container.as_ref());
        let mut storage = make_storage(container.as_ref());

        storage.delete().unwrap();

        assert_eq!(container.storages().unwrap().len(), 1);
    }

    #[rstest]
    fn create_storage_with_template_id(catlib: CatLib) {
        let container = _container(&catlib);
        make_storage(container.as_ref()); // Create storage w/o template id on purpose
        make_storage_with_template(container.as_ref(), Uuid::from_u128(1));
        make_storage_with_template(container.as_ref(), Uuid::from_u128(1));
        make_storage_with_template(container.as_ref(), Uuid::from_u128(2));

        let storages = catlib
            .find_storages_with_template(&Uuid::from_u128(1))
            .unwrap();
        assert_eq!(storages.len(), 2);

        let storages = catlib
            .find_storages_with_template(&Uuid::from_u128(2))
            .unwrap();
        assert_eq!(storages.len(), 1);
    }

    #[rstest]
    fn create_storage_with_data(container: Box<dyn Container>) {
        container
            .create_storage(None, b"storage data".to_vec())
            .unwrap();

        assert_eq!(
            (*container.storages().unwrap()[0]).as_ref().data,
            b"storage data"
        )
    }
}
