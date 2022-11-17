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

pub(crate) fn fetch_forest_by_uuid(_uuid: &Uuid) -> CatlibResult<Box<dyn IForest>> {
    // let forest: Vec<_> = data
    //     .iter()
    //     .filter(|(id, _)| id.starts_with(format!("forest-{uuid}").as_str()))
    //     .map(|(_, forest_str)| Forest::from_db_entry(forest_str, db.clone()))
    //     .collect();

    // match forest.len() {
    //     0 => Err(CatlibError::NoRecordsFound),
    //     1 => Ok(Box::new(forest[0].clone())),
    //     _ => Err(CatlibError::MalformedDatabaseRecord),
    // }
    todo!()
}

pub(crate) fn fetch_container_by_uuid(_uuid: &Uuid) -> CatlibResult<Box<dyn IContainer>> {
    todo!()
    // db.load().map_err(to_catlib_error)?;
    // let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    // let container: Vec<_> = data
    //     .iter()
    //     .filter(|(id, _)| id.starts_with(format!("container-{uuid}").as_str()))
    //     .map(|(_, container_str)| Container::from_db_entry(container_str, db.clone()))
    //     .collect();

    // match container.len() {
    //     0 => Err(CatlibError::NoRecordsFound),
    //     1 => Ok(Box::new(container[0].clone())),
    //     _ => Err(CatlibError::MalformedDatabaseRecord),
    // }
}

pub(crate) fn fetch_storages_by_container_uuid(
    _uuid: &Uuid,
) -> CatlibResult<Vec<Box<dyn IStorage>>> {
    todo!()
    // db.load().map_err(to_catlib_error)?;
    // let data = db.read(|db| db.clone()).map_err(to_catlib_error)?;

    // let storages: Vec<_> = data
    //     .iter()
    //     .filter(|(id, _)| id.starts_with("storage-"))
    //     .map(|(_, storage_str)| Storage::from_db_entry(storage_str, db.clone()))
    //     .filter(|storage| (*storage.container().unwrap()).as_ref().uuid == *uuid)
    //     .map(|storage| Box::new(storage) as Box<dyn IStorage>)
    //     .collect();

    // match storages.len() {
    //     0 => Err(CatlibError::NoRecordsFound),
    //     _ => Ok(storages),
    // }
}

pub(crate) fn _save_model(_key: String, _data: String) -> CatlibResult<()> {
    // db.load().map_err(to_catlib_error)?;

    // db.write(|db| db.insert(key, data))
    //     .map_err(to_catlib_error)?;

    // db.save().map_err(to_catlib_error)
    todo!()
}

pub(crate) fn _delete_model(_key: String) -> CatlibResult<()> {
    // db.load().map_err(to_catlib_error)?;

    // db.write(|db| db.remove_entry(&key))
    //     .map_err(to_catlib_error)?;

    // db.save().map_err(to_catlib_error)
    todo!()
}

#[cfg(test)]
pub(crate) mod test {

    use rstest::fixture;

    #[fixture]
    pub fn catlib() -> crate::CatLib {
        let random = rand::random::<uuid::Bytes>();
        let uuid = uuid::Builder::from_random_bytes(random).into_uuid();
        let dir = tempfile::tempdir().unwrap().into_path();
        let path = dir.join(format!("{uuid}-db.yaml"));
        crate::CatLib::new(path)
    }
}
