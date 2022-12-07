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

//! Catalog client library
//!
//! This library is used by Wildland Core to allow persistent storage for Wildland manifests that
//! describe Wildland entities such as Containers, Storages, Bridges etc.
//!
//! The library acts as a database client depending on the database backend used. The current
//! version of CatLib stores manifests in a local single-file nosql, unstructured database.
//! Location of the database file depends on the platform where the application runs, these are:
//!
//! - `Linux:   /home/alice/.config/catlib`
//! - `Windows: C:\Users\Alice\AppData\Roaming\com.wildland.Cargo\catlib`
//! - `macOS:   /Users/Alice/Library/Application Support/com.wildland.Cargo/catlib`
//!
//! ## Entities relationship
//!
//! ```none
//! +------------+          +------------+
//! |   Forest   | -------> |   Bridge   |
//! +------------+          +------------+
//!       |
//!       |       +-------------+
//!       +-----> |  Container  |
//!               +-------------+
//!                      |
//!                      |       +-----------+
//!                      +-----> |  Storage  |
//!                              +-----------+
//! ```
//! ## Example usage
//!
//! ```rust
//! # use wildland_catlib::CatLib;
//! # use wildland_corex::entities::Identity;
//! # use wildland_corex::interface::CatLib as ICatLib;
//! # use std::collections::HashSet;
//! # use uuid::Uuid;
//! let forest_owner = Identity([1; 32]);
//! let signer = Identity([2; 32]);
//!
//! let catlib = CatLib::default();
//! let forest = catlib.create_forest(
//!                  forest_owner,
//!                  HashSet::from([signer]),
//!                  vec![],
//!              ).unwrap();
//!
//! let mut container = forest.create_container("container name".to_owned()).unwrap();
//! container.add_path("/foo/bar".to_string());
//! container.add_path("/bar/baz".to_string());
//!
//! let storage_template_id = Uuid::from_u128(1);
//! let storage_data = b"credentials_and_whatnot".to_vec();
//! container.create_storage(Some(storage_template_id), storage_data);
//! ```
//!

use bridge::Bridge;
pub use common::*;
use container::Container;
use db::*;
use directories::ProjectDirs;
use error::*;
use forest::{Forest, ForestData};
use rustbreak::PathDatabase;
use std::path::PathBuf;
use std::rc::Rc;
use storage::{Storage, StorageData};
use uuid::Uuid;
use wildland_corex::catlib_service::interface::CatLib as ICatLib;
use wildland_corex::entities::{
    Container as IContainer, Forest as IForest, Identity, Signers, Storage as IStorage,
};

mod bridge;
mod common;
mod container;
mod db;
mod error;
mod forest;
mod storage;

#[derive(Clone)]
pub struct CatLib {
    db: Rc<StoreDb>,
}

impl CatLib {
    pub fn new(path: PathBuf) -> Self {
        let db = PathDatabase::create_at_path(path.clone(), CatLibData::new());

        if db.is_err() {
            let path_str = path.to_str().unwrap();
            panic!("Could not create CatLib database at {path_str}");
        }

        CatLib {
            db: Rc::new(db.unwrap()),
        }
    }
}

impl ICatLib for CatLib {
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::interface::CatLib as ICatLib;
    /// # use std::collections::HashSet;
    /// let forest_owner = Identity([1; 32]);
    /// let signer = Identity([2; 32]);
    ///
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  forest_owner,
    ///                  HashSet::from([signer]),
    ///                  vec![],
    ///              ).unwrap();
    /// ```
    #[tracing::instrument(level = "debug", skip_all)]
    fn create_forest(
        &self,
        owner: Identity,
        signers: Signers,
        data: Vec<u8>,
    ) -> CatlibResult<Box<dyn IForest>> {
        let forest = Box::new(Forest::new(owner, signers, data, self.db.clone()));
        forest.save()?;
        Ok(forest)
    }

    fn get_forest(&self, uuid: &Uuid) -> CatlibResult<Box<dyn IForest>> {
        fetch_forest_by_uuid(self.db.clone(), uuid)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Forest`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_forest(&self, owner: &Identity) -> CatlibResult<Box<dyn IForest>> {
        self.db.load().map_err(to_catlib_error)?;
        let data = self.db.read(|db| db.clone()).map_err(to_catlib_error)?;

        let forests: Vec<_> = data
            .iter()
            .filter(|(id, _)| id.starts_with("forest-"))
            .map(|(_, forest_str)| Forest {
                data: ForestData::from(forest_str.as_str()),
                db: self.db.clone(),
            })
            .filter(|forest| &forest.owner() == owner)
            .collect();

        match forests.len() {
            0 => Err(CatlibError::NoRecordsFound),
            1 => Ok(Box::new(forests[0].clone())),
            _ => Err(CatlibError::MalformedDatabaseRecord),
        }
    }

    fn get_container(&self, uuid: &Uuid) -> CatlibResult<Box<dyn IContainer>> {
        fetch_container_by_uuid(self.db.clone(), uuid)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Storage`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_storages_with_template(
        &self,
        template_id: &Uuid,
    ) -> CatlibResult<Vec<Box<dyn IStorage>>> {
        self.db.load().map_err(to_catlib_error)?;
        let data = self.db.read(|db| db.clone()).map_err(to_catlib_error)?;

        let storages: Vec<_> = data
            .iter()
            .filter(|(id, _)| id.starts_with("storage-"))
            .map(|(_, storage_str)|  Storage {
                data: StorageData::from(storage_str.as_str()),
                db: self.db.clone(),
            })
            .filter(
                |storage| matches!(storage.as_ref().template_uuid, Some(val) if val == *template_id),
            )
            .map(|storage| Box::new(storage) as Box<dyn IStorage>)
            .collect();

        match storages.len() {
            0 => Err(CatlibError::NoRecordsFound),
            _ => Ok(storages),
        }
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_containers_with_template(
        &self,
        template_id: &Uuid,
    ) -> CatlibResult<Vec<Box<dyn IContainer>>> {
        let storages = self.find_storages_with_template(template_id)?;
        storages.iter().map(|storage| storage.container()).collect()
    }
}

impl Default for CatLib {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("com", "wildland", "Cargo");

        let db_file = if let Some(project_dirs) = project_dirs {
            let db_dir = project_dirs.data_local_dir().join("catlib");

            if !db_dir.exists() {
                std::fs::create_dir_all(&db_dir).unwrap();
            }

            db_dir.join("catlib.database")
        } else {
            tracing::info!("Could not create ProjectDirs. Using working directory.");
            "./catlib.database".into()
        };

        CatLib {
            db: Rc::new(PathDatabase::load_from_path_or_default(db_file).unwrap()),
        }
    }
}
