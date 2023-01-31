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
use wildland_corex::catlib_service::entities::{
    ContainerPath,
    ContainerPaths,
    ForestManifest,
    StorageManifest,
};

use super::*;
use crate::storage::StorageEntity;

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

impl From<&ContainerEntity> for String {
    fn from(value: &ContainerEntity) -> Self {
        ron::to_string(&value.container_data).unwrap()
    }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) struct ContainerEntity {
    pub(crate) container_data: ContainerData,
    #[derivative(Debug = "ignore")]
    pub(crate) db: RedisDb,
}

impl ContainerEntity {
    pub fn from_container_data(container_data: ContainerData, db: &RedisDb) -> Self {
        Self {
            container_data,
            db: db.clone(),
        }
    }
}

impl ContainerManifest for ContainerEntity {
    fn add_path(&mut self, path: ContainerPath) -> Result<bool, CatlibError> {
        self.sync()?;
        if self.container_data.paths.contains(&path) {
            Ok(false)
        } else {
            self.container_data.paths.push(path);
            self.save()?;
            Ok(true)
        }
    }

    fn delete_path(&mut self, path: ContainerPath) -> Result<bool, CatlibError> {
        self.sync()?;
        if let Some(pos) = self.container_data.paths.iter().position(|p| *p == path) {
            self.container_data.paths.remove(pos);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_storages(&mut self) -> Result<Vec<Arc<Mutex<dyn StorageManifest>>>, CatlibError> {
        fetch_storages_by_container_uuid(&self.db, &self.container_data.uuid)
    }

    fn add_storage(
        &mut self,
        template_uuid: Uuid,
        serialized_storage: Vec<u8>,
    ) -> Result<Arc<Mutex<dyn StorageManifest>>, CatlibError> {
        let storage_entity = StorageEntity::new(
            self.container_data.uuid,
            Some(template_uuid),
            serialized_storage,
            &self.db,
        );
        storage_entity.save()?;
        self.sync()?;
        let storage = Arc::new(Mutex::new(storage_entity));
        Ok(storage)
    }

    fn stringify(&self) -> String {
        format!("{self:?}")
    }

    fn set_name(&mut self, new_name: String) -> CatlibResult<()> {
        self.sync()?;
        self.container_data.name = new_name;
        self.save()
    }

    fn name(&mut self) -> Result<String, CatlibError> {
        self.sync()?;
        Ok(self.container_data.name.clone())
    }

    fn remove(&mut self) -> Result<(), CatlibError> {
        self.delete()
    }

    fn forest(&self) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
        fetch_forest_by_uuid(&self.db, &self.container_data.forest_uuid)
    }

    fn uuid(&self) -> Uuid {
        self.container_data.uuid
    }

    fn get_paths(&mut self) -> Result<ContainerPaths, CatlibError> {
        self.sync()?;
        Ok(self.container_data.paths.clone())
    }

    fn owner(&self) -> Result<Identity, CatlibError> {
        let forest = fetch_forest_by_uuid(&self.db, &self.container_data.forest_uuid)?;
        let forest_lock = forest.lock().expect("Poisoned Mutex");
        Ok(forest_lock.owner())
    }

    fn serialise(&self) -> String {
        self.into()
    }
}

impl Model for ContainerEntity {
    fn save(&self) -> CatlibResult<()> {
        db::commands::set(
            &self.db,
            format!("container-{}", self.uuid()),
            self.serialise(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        db::commands::delete(&self.db, format!("container-{}", self.uuid()))
    }

    fn sync(&mut self) -> CatlibResult<()> {
        let container = db::fetch_container_by_uuid(&self.db, &self.uuid())?;
        let container_lock = container.lock().expect("Poisoned Mutex");
        self.container_data = ContainerData::from(container_lock.serialise().as_str());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

    use rstest::*;
    use wildland_corex::entities::ContainerManifest;

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

    fn make_container(catlib: &CatLib, container_uuid: Uuid) -> Arc<Mutex<dyn ContainerManifest>> {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();
        let forest_uuid: Uuid = forest.lock().unwrap().uuid();
        let name: String = "container_name".to_owned();
        let path = "/some/path".into();
        let forest_lock = forest.lock().unwrap();
        forest_lock
            .create_container(container_uuid, forest_uuid, name, path)
            .unwrap()
    }

    #[rstest(catlib_with_forest as catlib)]
    fn fetch_created_container(catlib: CatLib) {
        let container_uuid: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let container = make_container(&catlib, container_uuid);
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
        let container_uuid: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let container = make_container(&catlib, container_uuid);
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

        let container_uuid: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let container = make_container(&catlib, container_uuid);
        container
            .lock()
            .unwrap()
            .add_path("/foo/bar".into())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz".into())
            .unwrap();

        assert!(container
            .lock()
            .unwrap()
            .get_paths()
            .unwrap()
            .contains(&"/foo/bar".into()));
        assert!(container
            .lock()
            .unwrap()
            .get_paths()
            .unwrap()
            .contains(&"/bar/baz".into()));

        // Try to find that container in the database
        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/foo/bar".into()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);

        // Ensure again that it still has the paths
        assert!(container
            .lock()
            .unwrap()
            .get_paths()
            .unwrap()
            .contains(&"/foo/bar".into()));
        assert!(container
            .lock()
            .unwrap()
            .get_paths()
            .unwrap()
            .contains(&"/bar/baz".into()));

        // Try to fetch the same (one) container, using two different paths. The result
        // should be only one (not two) containers.
        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/foo/bar".into(), "/bar/baz".into()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);
    }

    #[rstest(catlib_with_forest as catlib)]
    fn multiple_containers_with_paths(catlib: CatLib) {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        let container_uuid1: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let container = make_container(&catlib, container_uuid1);
        container
            .lock()
            .unwrap()
            .add_path("/foo/bar".into())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz".into())
            .unwrap();

        // Create another container, that shares a path with the former
        let container_uuid2: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        let container = make_container(&catlib, container_uuid2);
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz".into())
            .unwrap();

        // And yet another container that doesn't
        let container_uuid3: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000003").unwrap();
        let container = make_container(&catlib, container_uuid3);
        container
            .lock()
            .unwrap()
            .add_path("/what/ever".into())
            .unwrap();

        // try to find the first container
        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/foo/bar".into()], false)
            .unwrap();
        assert_eq!(containers.len(), 1);

        // try to find the first and the second containers, using shared path
        let containers = forest
            .lock()
            .unwrap()
            .find_containers(vec!["/bar/baz".into()], false)
            .unwrap();
        assert_eq!(containers.len(), 2);

        // Make sure that they are in fact two different containers
        assert_ne!(
            containers[0].lock().unwrap().uuid(),
            containers[1].lock().unwrap().uuid()
        );
    }

    #[rstest(catlib_with_forest as catlib)]
    fn create_containers_with_different_storages(catlib: CatLib) {
        let container_uuid1 = Uuid::from_str("00000000-0000-0000-0000-000000000011").unwrap();
        let container_uuid2 = Uuid::from_str("00000000-0000-0000-0000-000000000012").unwrap();

        let alpha = make_container(&catlib, container_uuid1);
        let beta = make_container(&catlib, container_uuid2);

        let storage_uuid1 = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let storage_uuid2 = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        alpha
            .lock()
            .unwrap()
            .add_storage(storage_uuid1, vec![])
            .unwrap();
        alpha
            .lock()
            .unwrap()
            .add_storage(storage_uuid2, vec![])
            .unwrap();

        beta.lock()
            .unwrap()
            .add_storage(storage_uuid1, vec![])
            .unwrap();

        let containers = catlib
            .find_containers_with_template(&storage_uuid2)
            .unwrap();

        assert_eq!(containers.len(), 1);
        assert_eq!(
            containers[0].lock().unwrap().uuid(),
            alpha.lock().unwrap().uuid()
        );

        let containers = catlib
            .find_containers_with_template(&storage_uuid1)
            .unwrap();

        assert_eq!(containers.len(), 2);
        assert_ne!(
            containers[0].lock().unwrap().uuid(),
            containers[1].lock().unwrap().uuid()
        );
    }

    #[rstest(catlib_with_forest as catlib)]
    fn multiple_containers_with_subpaths(catlib: CatLib) {
        let forest = catlib.find_forest(&Identity([1; 32])).unwrap();

        let container_uuid1: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let container = make_container(&catlib, container_uuid1);
        container
            .lock()
            .unwrap()
            .add_path("/foo/bar1".into())
            .unwrap();

        let container_uuid2: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        let container = make_container(&catlib, container_uuid2);
        container
            .lock()
            .unwrap()
            .add_path("/foo/bar2".into())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz1".into())
            .unwrap();

        let container_uuid3: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000003").unwrap();
        let container = make_container(&catlib, container_uuid3);
        container
            .lock()
            .unwrap()
            .add_path("/bar/baz2".into())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/baz/qux1".into())
            .unwrap();
        container
            .lock()
            .unwrap()
            .add_path("/baz/qux2".into())
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
