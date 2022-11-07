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

use wildland_corex::entities::{Container as IContainer, Forest as IForest, Storage as IStorage};

use super::*;
use crate::error::to_catlib_error;
use std::rc::Rc;

pub(crate) fn fetch_forest_by_uuid(db: Rc<StoreDb>, uuid: &Uuid) -> CatlibResult<Box<dyn IForest>> {
    db.load().map_err(to_catlib_error)?;
    let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    let forest: Vec<_> = data
        .iter()
        .filter(|(id, _)| id.starts_with(format!("forest-{uuid}").as_str()))
        .map(|(_, forest_str)| forest_str.parse::<Forest>().unwrap())
        .collect();

    match forest.len() {
        0 => Err(CatlibError::NoRecordsFound),
        1 => Ok(Box::new(forest[0].clone())),
        _ => Err(CatlibError::MalformedDatabaseRecord),
    }
}

pub(crate) fn fetch_container_by_uuid(
    db: Rc<StoreDb>,
    uuid: &Uuid,
) -> CatlibResult<Box<dyn IContainer>> {
    db.load().map_err(to_catlib_error)?;
    let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    let container: Vec<_> = data
        .iter()
        .filter(|(id, _)| id.starts_with(format!("container-{uuid}").as_str()))
        .map(|(_, forest_str)| forest_str.parse::<Container>().unwrap())
        .collect();

    match container.len() {
        0 => Err(CatlibError::NoRecordsFound),
        1 => Ok(Box::new(container[0].clone())),
        _ => Err(CatlibError::MalformedDatabaseRecord),
    }
}

pub(crate) fn fetch_storages_by_container_uuid(
    db: Rc<StoreDb>,
    uuid: &Uuid,
) -> CatlibResult<Vec<Box<dyn IStorage>>> {
    db.load().map_err(to_catlib_error)?;
    let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    let storages: Vec<_> = data
        .iter()
        .filter(|(id, _)| id.starts_with("storage-"))
        .map(|(_, storage_str)| storage_str.parse::<Storage>().unwrap())
        .filter(|storage| (*storage.container().unwrap()).as_ref().uuid == *uuid)
        .map(|storage| Box::new(storage) as Box<dyn IStorage>)
        .collect();

    match storages.len() {
        0 => Err(CatlibError::NoRecordsFound),
        _ => Ok(storages),
    }
}

pub(crate) fn save_model(db: Rc<StoreDb>, key: String, data: String) -> CatlibResult<()> {
    db.load().map_err(to_catlib_error)?;

    db.write(|db| db.insert(key, data))
        .map_err(to_catlib_error)?;

    db.save().map_err(to_catlib_error)
}

pub(crate) fn delete_model(db: Rc<StoreDb>, key: String) -> CatlibResult<()> {
    db.load().map_err(to_catlib_error)?;

    db.write(|db| db.remove_entry(&key))
        .map_err(to_catlib_error)?;

    db.save().map_err(to_catlib_error)
}

#[cfg(test)]
pub(crate) fn init_catlib(random: uuid::Bytes) -> crate::CatLib {
    use mocktopus::mocking::*;

    let uuid = uuid::Builder::from_random_bytes(random).into_uuid();
    let dir = tempfile::tempdir().unwrap().into_path();

    let path = dir.join(format!("{uuid}-db.ron"));

    // Mock the `use_default_database` function to return a unique path to a file database
    // for each test. This allows not only to avoid having to mock whole Catlib structs to use
    // a different DB deserializer, but also allows tests to run in parallel due to the fact
    // that every test will have it's own, random UUID.
    crate::use_default_database.mock_safe(move || {
        let path = dir.join(format!("{uuid}-db.ron"));
        println!("{}", path.to_str().unwrap());
        let db = CatLib::new(path).db;
        db.load().unwrap();

        MockResult::Return(db)
    });

    CatLib::new(path)
}
