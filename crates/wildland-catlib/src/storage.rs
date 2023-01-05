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

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) struct StorageEntity {
    pub(crate) data: StorageData,

    #[derivative(Debug = "ignore")]
    pub(crate) db: Rc<StoreDb>,
}

impl StorageEntity {
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

impl AsRef<StorageData> for StorageEntity {
    fn as_ref(&self) -> &StorageData {
        &self.data
    }
}

impl StorageManifest for StorageEntity {
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Container`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn container(&self) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>> {
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

    fn uuid(&self) -> Uuid {
        self.data.uuid
    }
}

impl Model for StorageEntity {
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
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};

    use rstest::*;
    use wildland_corex::catlib_service::entities::{ContainerManifest, StorageManifest};
    use wildland_corex::StorageTemplate;

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
        let path = "/some/path".to_owned();
        locked_forest
            .create_container("name".to_owned(), &storage_template, path)
            .unwrap()
    }

    #[fixture]
    fn container(catlib_with_forest: CatLib) -> Arc<Mutex<dyn ContainerManifest>> {
        _container(&catlib_with_forest)
    }

    fn make_storage(
        container: &Arc<Mutex<dyn ContainerManifest>>,
    ) -> Arc<Mutex<dyn StorageManifest>> {
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
        container
            .lock()
            .unwrap()
            .add_storage(&storage_template)
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
        storage_template: &StorageTemplate,
    ) -> Arc<Mutex<dyn StorageManifest>> {
        container
            .lock()
            .unwrap()
            .add_storage(storage_template)
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
        make_storage(&container);

        assert_eq!(container.lock().unwrap().get_storages().unwrap().len(), 2);

        make_storage(&container);

        assert_eq!(container.lock().unwrap().get_storages().unwrap().len(), 3);
    }

    #[rstest]
    fn delete_a_storage(container: Arc<Mutex<dyn ContainerManifest>>) {
        make_storage(&container);
        let storage = make_storage(&container);

        storage.lock().unwrap().delete().unwrap();

        assert_eq!(container.lock().unwrap().get_storages().unwrap().len(), 2);
    }

    #[rstest(catlib_with_forest as catlib)]
    fn create_storage_with_template_id(catlib: CatLib) {
        let container = _container(&catlib);
        make_storage(&container); // Create storage w/o template id on purpose
        let storage_template1 = StorageTemplate::try_new(
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
        let storage_template2 = StorageTemplate::try_new(
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
        make_storage_with_template(&container, &storage_template1);
        make_storage_with_template(&container, &storage_template1);
        make_storage_with_template(&container, &storage_template2);

        let storages = catlib
            .find_storages_with_template(&storage_template1.uuid())
            .unwrap();
        assert_eq!(storages.len(), 2);

        let storages = catlib
            .find_storages_with_template(&storage_template2.uuid())
            .unwrap();
        assert_eq!(storages.len(), 1);
    }
}
