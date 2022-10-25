//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;
use std::rc::Rc;

pub(crate) fn fetch_forest_by_uuid(db: Rc<StoreDb>, uuid: Uuid) -> CatlibResult<Forest> {
    db.load()?;
    let data = db.read(|db| db.clone()).map_err(CatlibError::from)?;

    let forest: Vec<Forest> = data
        .iter()
        .filter(|(id, _)| (**id).starts_with(format!("forest-{uuid}").as_str()))
        .map(|(_, forest_str)| Forest::try_from((*forest_str).clone()).unwrap())
        .collect();

    match forest.len() {
        0 => Err(CatlibError::NoRecordsFound),
        1 => Ok(forest[0].clone()),
        _ => Err(CatlibError::MalformedDatabaseEntry),
    }
}

pub(crate) fn fetch_container_by_uuid(db: Rc<StoreDb>, uuid: Uuid) -> CatlibResult<Container> {
    db.load()?;
    let data = db.read(|db| db.clone()).map_err(CatlibError::from)?;

    let container: Vec<Container> = data
        .iter()
        .filter(|(id, _)| (**id).starts_with(format!("container-{uuid}").as_str()))
        .map(|(_, forest_str)| Container::try_from((*forest_str).clone()).unwrap())
        .collect();

    match container.len() {
        0 => Err(CatlibError::NoRecordsFound),
        1 => Ok(container[0].clone()),
        _ => Err(CatlibError::MalformedDatabaseEntry),
    }
}

pub(crate) fn fetch_storages_by_container_uuid(
    db: Rc<StoreDb>,
    uuid: Uuid,
) -> CatlibResult<Vec<Storage>> {
    db.load()?;
    let data = db.read(|db| db.clone()).map_err(CatlibError::from)?;

    let storages: Vec<Storage> = data
        .iter()
        .filter(|(id, _)| (**id).starts_with("storage-"))
        .map(|(_, storage_str)| Storage::try_from((*storage_str).clone()).unwrap())
        .filter(|storage| storage.container().unwrap().uuid() == uuid)
        .collect();

    match storages.len() {
        0 => Err(CatlibError::NoRecordsFound),
        _ => Ok(storages),
    }
}

pub(crate) fn save_model(db: Rc<StoreDb>, key: String, data: String) -> CatlibResult<()> {
    db.load()?;

    db.write(|db| db.insert(key, data))
        .map_err(CatlibError::from)?;

    db.save().map_err(CatlibError::from)
}

pub(crate) fn delete_model(db: Rc<StoreDb>, key: String) -> CatlibResult<()> {
    db.load()?;

    db.write(|db| db.remove_entry(&key))
        .map_err(CatlibError::from)?;

    db.save().map_err(CatlibError::from)
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
