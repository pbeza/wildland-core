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
//! # use std::collections::HashSet;
//! # use crate::wildland_catlib::*;
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
//! let mut container = forest.create_container().unwrap();
//! container.add_path("/foo/bar".to_string());
//! container.add_path("/bar/baz".to_string());
//!
//! let storage_template_id = Uuid::from_u128(1);
//! let storage_data = b"credentials_and_whatnot".to_vec();
//! container.create_storage(Some(storage_template_id), storage_data);
//! ```
//!

#![cfg_attr(test, feature(proc_macro_hygiene))]

pub use bridge::Bridge;
pub use container::Container;
pub use contracts::common::*;
pub use contracts::*;
use db::*;
use directories::ProjectDirs;
pub use error::*;
pub use forest::Forest;
use rustbreak::PathDatabase;
use std::path::PathBuf;
use std::rc::Rc;
pub use storage::Storage;
use uuid::Uuid;

mod bridge;
mod container;
pub mod contracts;
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

    /// Create new Forest obect.
    ///
    /// `owner` and `signers` are cryptographical objects that are used by the Core module to
    /// verify the cryptographical integrity of the manifests.
    ///
    /// `data` is an arbitrary data object that can be used to synchronize Forest state between
    /// devices.
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::{CatLib, Identity};
    /// # use std::collections::HashSet;
    /// # use crate::wildland_catlib::IForest;
    /// let forest_owner = Identity([1; 32]);
    /// let signer = Identity([2; 32]);
    ///
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  forest_owner,
    ///                  HashSet::from([signer]),
    ///                  vec![],
    ///              ).unwrap();
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

    /// Return [`Forest`] object by Forest UUID.
    pub fn get_forest(&self, uuid: Uuid) -> CatlibResult<Forest> {
        fetch_forest_by_uuid(self.db.clone(), uuid)
    }

    /// Return [`Forest`] owned by specified `owner`.
    ///
    /// **Note: by design each owner may have only one Forest**
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Forest`] was found.
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

    /// Return [`Container`] object by Container UUID.
    pub fn get_container(&self, uuid: Uuid) -> CatlibResult<Container> {
        fetch_container_by_uuid(self.db.clone(), uuid)
    }

    /// Return [`Storage`]s that were created using given `template_id` UUID.
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Forest`] was found.
    pub fn find_storages_with_template(&self, template_id: Uuid) -> CatlibResult<Vec<Storage>> {
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

    /// Return [`Container`]s that were created using given `template_id` UUID.
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Forest`] was found.
    pub fn find_containers_with_template(&self, template_id: Uuid) -> CatlibResult<Vec<Container>> {
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

#[cfg_attr(test, mocktopus::macros::mockable)]
fn use_default_database() -> Rc<StoreDb> {
    let db = CatLib::default().db;
    db.load().unwrap(); // Definitely not thread safe
    db
}
