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
//! ```no_run
//! # use wildland_catlib::CatLib;
//! # use wildland_corex::entities::Identity;
//! # use wildland_corex::interface::CatLib as ICatLib;
//! # use wildland_corex::StorageTemplate;
//! # use std::collections::{HashSet, HashMap};
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
//! let storage_template = StorageTemplate::try_new(
//!     "FoundationStorage",
//!     HashMap::from([
//!             (
//!                 "field1".to_owned(),
//!                 "Some value with container name: {{ CONTAINER_NAME }}".to_owned(),
//!             ),
//!             (
//!                 "parameter in key: {{ OWNER }}".to_owned(),
//!                 "enum: {{ ACCESS_MODE }}".to_owned(),
//!             ),
//!             ("uuid".to_owned(), "{{ CONTAINER_UUID }}".to_owned()),
//!             ("paths".to_owned(), "{{ PATHS }}".to_owned()),
//!         ]),
//!     )
//!     .unwrap();
//! let path = "/some/path".to_owned();
//! let container = forest.lock().unwrap().create_container("container name".to_owned(), &storage_template, path).unwrap();
//! container.lock().unwrap().add_path("/foo/bar".to_string());
//! container.lock().unwrap().add_path("/bar/baz".to_string());
//!
//! ```
//!

use std::path::PathBuf;
use std::rc::Rc;

use bridge::Bridge;
pub use common::*;
use container::Container;
use db::*;
use directories::ProjectDirs;
use error::*;
use forest::{Forest, ForestData};
use rustbreak::PathDatabase;
use storage::{Storage, StorageData};
use uuid::Uuid;
use wildland_corex::catlib_service::entities::{
    ContainerManifest as IContainer,
    ForestManifest as IForest,
    Identity,
    Signers,
    StorageManifest as IStorage,
};
use wildland_corex::catlib_service::interface::CatLib as ICatLib;

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
    /// # use wildland_corex::catlib_service::entities::Identity;
    /// # use wildland_corex::catlib_service::interface::CatLib as ICatLib;
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
    ) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
        let forest = Forest::new(owner, signers, data, self.db.clone());
        forest.save()?;
        Ok(Arc::new(Mutex::new(forest)))
    }

    fn get_forest(&self, uuid: &Uuid) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
        fetch_forest_by_uuid(self.db.clone(), uuid)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Forest`] was found.
    /// - Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_forest(&self, owner: &Identity) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
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
            .map(|forest| Arc::new(Mutex::new(forest)))
            .collect();

        match forests.len() {
            0 => Err(CatlibError::NoRecordsFound),
            1 => Ok(forests[0].clone()),
            _ => Err(CatlibError::MalformedDatabaseRecord),
        }
    }

    fn get_container(&self, uuid: &Uuid) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>> {
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
    ) -> CatlibResult<Vec<Arc<Mutex<dyn StorageManifest>>>> {
        self.db.load().map_err(to_catlib_error)?;
        let data = self.db.read(|db| db.clone()).map_err(to_catlib_error)?;

        let storages: Vec<_> = data
            .iter()
            .filter(|(id, _)| id.starts_with("storage-"))
            .map(|(_, storage_str)| StorageEntity {
                data: StorageData::from(storage_str.as_str()),
                db: self.db.clone(),
            })
            .filter(
                |storage| matches!(storage.as_ref().template_uuid, Some(val) if val == *template_id),
            )
            .map(|storage| Arc::new(Mutex::new(storage)) as Arc<Mutex<dyn StorageManifest>>)
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
    ) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>> {
        let storages = self.find_storages_with_template(template_id)?;
        storages
            .iter()
            .map(|storage| {
                storage
                    .lock()
                    .map_err(|_| CatlibError::Generic("Poisoned Mutex".to_owned()))?
                    .container()
            })
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn save_storage_template(&self, template_id: &Uuid, value: String) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("template-storage-{template_id}"),
            value,
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_storage_templates_data(&self) -> CatlibResult<Vec<String>> {
        self.db.load().map_err(to_catlib_error)?;
        let data = self.db.read(|db| db.clone()).map_err(to_catlib_error)?;

        let storages: Vec<_> = data
            .iter()
            .filter(|(id, _)| id.starts_with("template-storage-"))
            .map(|(_, storage_str)| storage_str)
            .cloned()
            .collect();
        Ok(storages)
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
