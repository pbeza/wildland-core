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

use wildland_corex::catlib_service::entities::{
    ContainerManifest as IContainer,
    ForestManifest as IForest,
    StorageManifest as IStorage,
};

use super::*;
use crate::bridge::BridgeData;
use crate::container::ContainerData;
use crate::error::to_catlib_error;
use crate::forest::ForestData;
use crate::storage::StorageData;

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_forest_by_uuid(
    db: Rc<StoreDb>,
    uuid: &Uuid,
) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
    let data = fetch_forest_data_by_uuid(db.clone(), uuid)?;
    let forest = Forest { data, db };
    Ok(Arc::new(Mutex::new(forest)))
}

pub(crate) fn fetch_forest_data_by_uuid(db: Rc<StoreDb>, uuid: &Uuid) -> CatlibResult<ForestData> {
    db.load().map_err(to_catlib_error)?;
    let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    let forest: Vec<_> = data
        .iter()
        .filter(|(id, _)| id.starts_with(format!("forest-{uuid}").as_str()))
        .map(|(_, forest_str)| ForestData::from(forest_str.as_str()))
        .collect();

    match forest.len() {
        0 => Err(CatlibError::NoRecordsFound),
        1 => Ok(forest[0].clone()),
        _ => Err(CatlibError::MalformedDatabaseRecord),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_container_by_uuid(
    db: Rc<StoreDb>,
    uuid: &Uuid,
) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>> {
    let container_data = fetch_container_data_by_uuid(db.clone(), uuid)?;
    Container::from_container_data(container_data, db)
        .map(|c| Arc::new(Mutex::new(c)) as Arc<Mutex<dyn ContainerManifest>>)
}

pub(crate) fn fetch_container_data_by_uuid(
    db: Rc<StoreDb>,
    uuid: &Uuid,
) -> CatlibResult<ContainerData> {
    db.load().map_err(to_catlib_error)?;
    let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    let container: Vec<_> = data
        .iter()
        .filter(|(id, _)| id.starts_with(format!("container-{uuid}").as_str()))
        .map(|(_, container_str)| ContainerData::from(container_str.as_str()))
        .collect();

    match container.len() {
        0 => Err(CatlibError::NoRecordsFound),
        1 => Ok(container[0].clone()),
        _ => Err(CatlibError::MalformedDatabaseRecord),
    }
}

pub(crate) fn fetch_storage_data_by_uuid(
    db: Rc<StoreDb>,
    uuid: &Uuid,
) -> CatlibResult<StorageData> {
    db.load().map_err(to_catlib_error)?;
    let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    let storages: Vec<_> = data
        .iter()
        .filter(|(id, _)| id.starts_with(&format!("storage-{uuid}")))
        .map(|(_, storage_str)| StorageData::from(storage_str.as_str()))
        .collect();

    match storages.len() {
        0 => Err(CatlibError::NoRecordsFound),
        1 => Ok(storages[0].clone()),
        _ => Err(CatlibError::MalformedDatabaseRecord),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_storages_by_container_uuid(
    db: Rc<StoreDb>,
    uuid: &Uuid,
) -> CatlibResult<Vec<Arc<Mutex<dyn StorageManifest>>>> {
    db.load().map_err(to_catlib_error)?;
    let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    let storages: Vec<_> = data
        .iter()
        .filter(|(id, _)| id.starts_with("storage-"))
        .map(|(_, storage_str)| StorageData::from(storage_str.as_str()))
        .filter(|storage_data| storage_data.container_uuid == *uuid)
        .map(|storage_data| StorageEntity {
            data: storage_data,
            db: db.clone(),
        })
        .map(|storage| Arc::new(Mutex::new(storage)) as Arc<Mutex<dyn StorageManifest>>)
        .collect();

    match storages.len() {
        0 => Err(CatlibError::NoRecordsFound),
        _ => Ok(storages),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_bridge_data_by_uuid(db: Rc<StoreDb>, uuid: &Uuid) -> CatlibResult<BridgeData> {
    db.load().map_err(to_catlib_error)?;
    let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    let bridges: Vec<_> = data
        .iter()
        .filter(|(id, _)| id.starts_with(&format!("bridge-{uuid}")))
        .map(|(_, bridge_str)| BridgeData::from(bridge_str.as_str()))
        .collect();

    match bridges.len() {
        0 => Err(CatlibError::NoRecordsFound),
        1 => Ok(bridges[0].clone()),
        _ => Err(CatlibError::MalformedDatabaseRecord),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn save_model(db: Rc<StoreDb>, key: String, data: String) -> CatlibResult<()> {
    db.load().map_err(to_catlib_error)?;

    db.write(|db| db.insert(key, data))
        .map_err(to_catlib_error)?;

    db.save().map_err(to_catlib_error)
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn delete_model(db: Rc<StoreDb>, key: String) -> CatlibResult<()> {
    db.load().map_err(to_catlib_error)?;

    db.write(|db| db.remove_entry(&key))
        .map_err(to_catlib_error)?;

    db.save().map_err(to_catlib_error)
}

#[cfg(test)]
pub(crate) mod test {

    use rstest::fixture;

    #[fixture]
    pub fn catlib() -> crate::CatLib {
        let random = rand::random::<uuid::Bytes>();
        let uuid = uuid::Builder::from_random_bytes(random).into_uuid();
        let dir = tempfile::tempdir().unwrap().into_path();
        let path = dir.join(format!("{uuid}-db.ron"));
        crate::CatLib::new(path)
    }
}
