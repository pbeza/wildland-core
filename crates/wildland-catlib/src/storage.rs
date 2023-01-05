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
use wildland_corex::catlib_service::entities::{ContainerManifest, StorageManifest as IStorage};

use super::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct StorageData {
    pub uuid: Uuid,
    pub container_uuid: Uuid,
    pub template_uuid: Option<Uuid>,
    pub data: Vec<u8>,
}

impl From<&str> for StorageData {
    fn from(data_str: &str) -> Self {
        ron::from_str(data_str).unwrap()
    }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) struct Storage {
    pub(crate) data: StorageData,

    #[derivative(Debug = "ignore")]
    pub(crate) db: Rc<StoreDb>,
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
    fn container(&self) -> CatlibResult<Box<dyn ContainerManifest>> {
        fetch_container_by_uuid(self.db.clone(), &self.data.container_uuid)
    }

    /// ## Errors
    ///
    /// Retrieves Storage data
    fn data(&mut self) -> CatlibResult<Vec<u8>> {
        self.sync()?;
        Ok(self.data.data.clone())
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<()> {
        self.data.data = data;
        self.save()?;
        Ok(())
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
    fn save(&self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("storage-{}", self.data.uuid),
            ron::to_string(&self.data).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(self.db.clone(), format!("storage-{}", self.data.uuid))
    }

    fn sync(&mut self) -> CatlibResult<()> {
        let data = fetch_storage_data_by_uuid(self.db.clone(), &self.data.uuid)?;
        self.data = data;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rstest::*;
    use wildland_corex::catlib_service::entities::{ContainerManifest, StorageManifest};

    use super::db::test::catlib;
    use crate::*;

    #[fixture]
    fn catlib_with_forest(catlib: CatLib) -> CatLib {
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

    fn _container(catlib: &CatLib) -> Box<dyn ContainerManifest> {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();
        forest.create_container("name".to_owned()).unwrap()
    }

    #[fixture]
    fn container(catlib_with_forest: CatLib) -> Box<dyn ContainerManifest> {
        _container(&catlib_with_forest)
    }

    fn make_storage(container: &dyn ContainerManifest) -> Box<dyn StorageManifest> {
        container.create_storage(None, vec![]).unwrap()
    }

    fn make_storage_with_template(
        container: &dyn ContainerManifest,
        template_id: Uuid,
    ) -> Box<dyn StorageManifest> {
        container.create_storage(Some(template_id), vec![]).unwrap()
    }

    #[rstest]
    fn create_empty_storage(container: Box<dyn ContainerManifest>) {
        make_storage(container.as_ref());

        assert_eq!(container.storages().unwrap().len(), 1);

        make_storage(container.as_ref());

        assert_eq!(container.storages().unwrap().len(), 2);
    }

    #[rstest]
    fn delete_a_storage(container: Box<dyn ContainerManifest>) {
        make_storage(container.as_ref());
        let mut storage = make_storage(container.as_ref());

        storage.delete().unwrap();

        assert_eq!(container.storages().unwrap().len(), 1);
    }

    #[rstest(catlib_with_forest as catlib)]
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
    fn create_storage_with_data(container: Box<dyn ContainerManifest>) {
        container
            .create_storage(None, b"storage data".to_vec())
            .unwrap();

        assert_eq!(
            container.storages().unwrap()[0].data().unwrap(),
            b"storage data"
        )
    }
}
