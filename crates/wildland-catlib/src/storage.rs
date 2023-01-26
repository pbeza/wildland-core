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

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use wildland_corex::catlib_service::entities::{ContainerManifest, StorageManifest};

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

impl From<&StorageEntity> for String {
    fn from(value: &StorageEntity) -> Self {
        ron::to_string(&value.storage_data).unwrap()
    }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) struct StorageEntity {
    pub(crate) storage_data: StorageData,

    #[derivative(Debug = "ignore")]
    pub(crate) db: RedisDb,
}

impl StorageEntity {
    pub fn new(
        container_uuid: Uuid,
        template_uuid: Option<Uuid>,
        data: Vec<u8>,
        db: RedisDb,
    ) -> Self {
        Self {
            storage_data: StorageData {
                uuid: Uuid::new_v4(),
                container_uuid,
                template_uuid,
                data,
            },
            db,
        }
    }

    pub fn from_storage_data(storage_data: StorageData, db: RedisDb) -> Self {
        Self { storage_data, db }
    }
}

impl AsRef<StorageData> for StorageEntity {
    fn as_ref(&self) -> &StorageData {
        &self.storage_data
    }
}

impl StorageManifest for StorageEntity {
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Container`] was found.
    /// - Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    fn container(&self) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>> {
        fetch_container_by_uuid(self.db.clone(), &self.storage_data.container_uuid)
    }

    /// ## Errors
    ///
    /// Retrieves Storage data
    fn data(&mut self) -> CatlibResult<Vec<u8>> {
        self.sync()?;
        Ok(self.storage_data.data.clone())
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<()> {
        self.storage_data.data = data;
        self.save()?;
        Ok(())
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    fn remove(&mut self) -> CatlibResult<bool> {
        Model::delete(self)?;
        Ok(true)
    }

    fn uuid(&self) -> Uuid {
        self.storage_data.uuid
    }

    fn serialise(&self) -> String {
        self.into()
    }

    fn template_uuid(&self) -> Option<Uuid> {
        self.storage_data.template_uuid
    }
}

impl Model for StorageEntity {
    fn save(&self) -> CatlibResult<()> {
        db::commands::set(
            self.db.clone(),
            format!("storage-{}", self.uuid()),
            self.serialise(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        db::commands::delete(self.db.clone(), format!("storage-{}", self.uuid()))
    }

    fn sync(&mut self) -> CatlibResult<()> {
        let storage = db::fetch_storage_by_uuid(self.db.clone(), &self.uuid())?;
        let storage_lock = storage.lock().expect("Poisoned Mutex");
        self.storage_data = StorageData::from(storage_lock.serialise().as_str());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

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

    fn _container(catlib: &CatLib) -> Arc<Mutex<dyn ContainerManifest>> {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();
        let locked_forest = forest.lock().unwrap();
        let forest_uuid = locked_forest.uuid();
        let container_uuid = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let path = "/some/path".into();
        locked_forest
            .create_container(container_uuid, forest_uuid, "name".to_owned(), path)
            .unwrap()
    }

    #[fixture]
    fn container(catlib_with_forest: CatLib) -> Arc<Mutex<dyn ContainerManifest>> {
        _container(&catlib_with_forest)
    }

    fn make_storage(
        container: &Arc<Mutex<dyn ContainerManifest>>,
        storage_uuid: Uuid,
    ) -> Arc<Mutex<dyn StorageManifest>> {
        container
            .lock()
            .unwrap()
            .add_storage(storage_uuid, vec![])
            .unwrap();
        container
            .lock()
            .unwrap()
            .get_storages()
            .unwrap()
            .last()
            .unwrap()
            .clone()
    }

    fn make_storage_with_template(
        container: &Arc<Mutex<dyn ContainerManifest>>,
        serialized_storage: Vec<u8>,
        storage_uuid: Uuid,
    ) -> Arc<Mutex<dyn StorageManifest>> {
        container
            .lock()
            .unwrap()
            .add_storage(storage_uuid, serialized_storage)
            .unwrap();
        container
            .lock()
            .unwrap()
            .get_storages()
            .unwrap()
            .last()
            .unwrap()
            .clone()
    }

    #[rstest]
    fn create_empty_storage(container: Arc<Mutex<dyn ContainerManifest>>) {
        let storage_uuid1 = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let storage_uuid2 = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        make_storage(&container, storage_uuid1);

        assert_eq!(container.lock().unwrap().get_storages().unwrap().len(), 1);

        make_storage(&container, storage_uuid2);

        assert_eq!(container.lock().unwrap().get_storages().unwrap().len(), 2);
    }

    #[rstest]
    fn delete_a_storage(container: Arc<Mutex<dyn ContainerManifest>>) {
        let storage_uuid1 = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let storage_uuid2 = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        make_storage(&container, storage_uuid1);
        let storage = make_storage(&container, storage_uuid2);

        storage.lock().unwrap().remove().unwrap();

        assert_eq!(container.lock().unwrap().get_storages().unwrap().len(), 1);
    }

    #[rstest(catlib_with_forest as catlib)]
    fn create_storage_with_template_id(catlib: CatLib) {
        let container = _container(&catlib);
        let storage_uuid0 = Uuid::from_str("00000000-0000-0000-0000-000000000000").unwrap();
        make_storage(&container, storage_uuid0); // Create storage w/o template id on purpose
        let storage_uuid1 = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let storage_uuid2 = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        make_storage_with_template(&container, vec![], storage_uuid1);
        make_storage_with_template(&container, vec![], storage_uuid1);
        make_storage_with_template(&container, vec![], storage_uuid2);

        let storages = catlib.find_storages_with_template(&storage_uuid1).unwrap();
        assert_eq!(storages.len(), 2);

        let storages = catlib.find_storages_with_template(&storage_uuid2).unwrap();
        assert_eq!(storages.len(), 1);
    }
}
