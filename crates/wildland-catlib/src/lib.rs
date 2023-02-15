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
//! # use std::str::FromStr;
//! let forest_owner = Identity([1; 32]);
//! let signer = Identity([2; 32]);
//!
//! let catlib = CatLib::default();
//! let forest = catlib.create_forest(
//!                  forest_owner,
//!                  HashSet::from([signer]),
//!                  vec![],
//!              ).unwrap();
//! let container_uuid: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
//! let forest_uuid: Uuid = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
//! let name: String = "container_name".to_owned();
//! let path = "/some/path".into();
//! let container = forest.lock().unwrap().create_container(container_uuid, forest_uuid, name, path).unwrap();
//! container.lock().unwrap().add_path("/foo/bar".into());
//! container.lock().unwrap().add_path("/bar/baz".into());
//!
//! ```
//!

use std::sync::{Arc, Mutex};

use bridge::Bridge;
pub use common::*;
use container::ContainerEntity;
use db::*;
use error::*;
use forest::ForestEntity;
use storage::StorageEntity;
use uuid::Uuid;
use wildland_corex::catlib_service::entities::{
    ContainerManifest,
    ForestManifest,
    Identity,
    Signers,
    StorageManifest,
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
    db: RedisDb,
}

impl CatLib {
    pub fn new(redis_url: String, key_prefix: String) -> Self {
        let db = db::db_conn(redis_url.clone());

        if let Some(err) = db.clone().err() {
            panic!(
                "Could not instantiate Redis backend connection pool for the given URL [{redis_url}]. {err:?}",
            );
        }

        CatLib {
            db: RedisDb {
                client: db.unwrap(),
                key_prefix,
            },
        }
    }
}

impl ICatLib for CatLib {
    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
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
        let forest = ForestEntity::new(owner, signers, data, &self.db)?;
        Ok(Arc::new(Mutex::new(forest)))
    }

    fn get_forest(&self, uuid: &Uuid) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
        fetch_forest_by_uuid(&self.db, uuid)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Forest`] was found.
    /// - Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_forest(&self, owner: &Identity) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
        let forests = db::fetch_all_forests(&self.db)?;
        let forests: Vec<_> = forests
            .iter()
            .filter(|forest| &forest.lock().expect("Poisoned Mutex").owner() == owner)
            .collect();

        match forests.len() {
            0 => Err(CatlibError::NoRecordsFound),
            1 => Ok(forests[0].clone()),
            _ => Err(CatlibError::MalformedDatabaseRecord),
        }
    }

    fn get_container(&self, uuid: &Uuid) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>> {
        fetch_container_by_uuid(&self.db, uuid)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Storage`] was found.
    /// - Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_storages_with_template(
        &self,
        template_id: &Uuid,
    ) -> CatlibResult<Vec<Arc<Mutex<dyn StorageManifest>>>> {
        db::fetch_storages_by_template_uuid(&self.db, template_id)
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    /// - Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    fn find_containers_with_template(
        &self,
        template_id: &Uuid,
    ) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>> {
        let storages = self.find_storages_with_template(template_id)?;
        storages
            .iter()
            .map(|storage| storage.lock().expect("Poisoned Mutex").container())
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn save_storage_template(&self, template_id: &Uuid, value: String) -> CatlibResult<()> {
        db::commands::set(&self.db, format!("template-{template_id}"), value)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_storage_templates_data(&self) -> CatlibResult<Vec<String>> {
        db::fetch_templates(&self.db)
    }

    /// Checks if connection to the backend database is valid
    ///
    /// Be warned that Redis implementation may return generic IO errors
    /// instead of application-level errors.
    /// see: https://github.com/redis-rs/redis-rs/issues/784
    ///
    #[tracing::instrument(level = "debug", skip_all)]
    fn is_db_alive(&self) -> CatlibResult<bool> {
        db::is_alive(&self.db)
    }
}

impl Default for CatLib {
    fn default() -> Self {
        use std::env;
        let redis_url =
            env::var("CARGO_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".into());
        let db_prefix = env::var("CARGO_DB_KEY_PREFIX").unwrap_or_else(|_| "".into());

        CatLib::new(redis_url, db_prefix)
    }
}
