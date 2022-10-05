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

#![cfg_attr(test, feature(proc_macro_hygiene))]

pub use contracts::*;
pub use contracts::common::*;
pub use error::*;
pub use bridge::Bridge;
pub use container::Container;
pub use forest::Forest;
pub use storage::Storage;
use directories::ProjectDirs;
use rustbreak::deser::Ron;
use rustbreak::PathDatabase;
use std::path::PathBuf;
use std::rc::Rc;
use uuid::Uuid;
use db::*;

mod bridge;
mod container;
mod contracts;
mod db;
mod error;
mod forest;
mod storage;

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub type Identity = Vec<u8>;
type CatLibData = std::collections::HashMap<String, String>;
type StoreDb = PathDatabase<CatLibData, Ron>;

pub struct CatLib {
    db: Rc<StoreDb>,
}

impl CatLib {
    pub fn new(path: PathBuf) -> Self {
        let db = PathDatabase::create_at_path(path.clone(), CatLibData::new());

        if db.is_err() {
            let path_str = path.to_str().unwrap();
            panic!("Could not create CatLib database at {}", path_str);
        }

        CatLib {
            db: Rc::new(db.unwrap()),
        }
    }

    pub fn create_forest(
        &self,
        owner: Identity,
        signers: Signers,
        data: Vec<u8>,
    ) -> CatlibResult<Forest> {
        let mut forest = Forest::new(owner, signers, data, self.db.clone());
        forest.save()?;

        Ok(forest)
    }

    pub fn get_forest(&self, uuid: String) -> CatlibResult<Forest> {
        fetch_forest_by_uuid(self.db.clone(), uuid)
    }

    pub fn find_forest(&self, owner: Identity) -> CatlibResult<Forest> {
        self.db.load()?;
        let data = self.db.read(|db| db.clone()).map_err(CatlibError::from)?;

        let forests: Vec<Forest> = data
            .iter()
            .filter(|(id, _)| (**id).starts_with("forest-"))
            .map(|(_, forest_str)| Forest::try_from((*forest_str).clone()).unwrap())
            .filter(|forest| forest.owner() == owner)
            .collect();

        match forests.len() {
            0 => Err(CatlibError::NoRecordsFound),
            1 => Ok(forests[0].clone()),
            _ => Err(CatlibError::MalformedDatabaseEntry),
        }
    }

    pub fn get_container(&self, uuid: String) -> CatlibResult<Container> {
        fetch_container_by_uuid(self.db.clone(), uuid)
    }

    pub fn find_storages_with_template(&self, template_id: String) -> CatlibResult<Vec<Storage>> {
        self.db.load()?;
        let data = self.db.read(|db| db.clone()).map_err(CatlibError::from)?;

        let storages: Vec<Storage> = data
            .iter()
            .filter(|(id, _)| (**id).starts_with("storage-"))
            .map(|(_, storage_str)| Storage::try_from((*storage_str).clone()).unwrap())
            .filter(|storage| {
                storage.template_uuid().is_some() && storage.template_uuid().unwrap() == template_id
            })
            .collect();

        match storages.len() {
            0 => Err(CatlibError::NoRecordsFound),
            _ => Ok(storages),
        }
    }

    pub fn find_containers_with_template(
        &self,
        template_id: String,
    ) -> CatlibResult<Vec<Container>> {
        let storages = self.find_storages_with_template(template_id)?;

        storages.iter().map(|storage| storage.container()).collect()
    }
}

impl Default for CatLib {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("com", "wildland", "Cargo");

        if project_dirs.is_none() {
            panic!("Could not instantiate Catlib database directory");
        }

        let db_dir = project_dirs.unwrap().data_local_dir().join("catlib");

        if !db_dir.exists() {
            std::fs::create_dir_all(&db_dir).unwrap();
        }

        let db_file = db_dir.join("catlib.database");

        CatLib {
            db: Rc::new(PathDatabase::load_from_path_or_default(db_file).unwrap()),
        }
    }
}

#[cfg_attr(test, mocktopus::macros::mockable)]
fn use_default_database() -> Rc<StoreDb> {
    let db = CatLib::default().db;
    db.load().unwrap(); // Definitely not thread safe
    db
}
