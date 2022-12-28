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

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::CatlibResult;

pub type PubKey = [u8; 32];

impl From<Identity> for PubKey {
    fn from(identity: Identity) -> Self {
        identity.0
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct Identity(pub PubKey);

impl Identity {
    pub fn encode(&self) -> String {
        hex::encode(self.0)
    }
}

impl From<PubKey> for Identity {
    fn from(pubkey: [u8; 32]) -> Self {
        Self(pubkey)
    }
}

pub type ContainerPath = String;
pub type ContainerPaths = HashSet<ContainerPath>;
pub type Signers = HashSet<Identity>;

/// `ForestManifest` trait is an API providing methods needed to operate on the forest's
/// state. It should be implemented by Cat-Lib instance and should be
/// treated as a kind of a proxy layer between Wildland Core and the external
/// persistent data storage instance (for e.g. database).
///
#[cfg_attr(test, mockall::automock)]
pub trait ForestManifest: std::fmt::Debug {
    /// Add manifest Signer
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// - If the signer did not previously exist, `true` is returned.
    /// - If the signer already exists, `false` is returned.
    ///
    fn add_signer(&mut self, signer: Identity) -> Result<bool, CatlibError>;

    /// Delete manifest Signer
    ///
    /// Returns whether the value was already present. That is:
    ///
    /// - If the signer did not previously exist, `false` is returned.
    /// - If the signer existed in the set, `true` is returned.
    ///
    fn del_signer(&mut self, signer: Identity) -> Result<bool, CatlibError>;

    /// Return list of Forest Containers
    ///
    fn containers(&self) -> Result<Vec<Arc<Mutex<dyn ContainerManifest>>>, CatlibError>;

    /// Set Forest arbitrary data
    ///
    fn update(&mut self, data: Vec<u8>) -> Result<(), CatlibError>;

    /// Delete Forest
    ///
    /// **WARN: The underlying objects are not removed recursively**
    ///
    fn delete(&mut self) -> Result<bool, CatlibError>;

    /// Create an empty container, bound to the Forest.
    ///
    /// To set container paths, use [`Container::add_path`]
    ///
    fn create_container(
        &self,
        name: String,
        storage_data: &StorageTemplate,
        path: ContainerPath,
    ) -> Result<Arc<Mutex<dyn ContainerManifest>>, CatlibError>;

    /// Create a Bridge object with arbitrary link data to another Forest.
    ///
    /// The aforementioned link data will be defined by the D/FS module.
    ///
    fn create_bridge(
        &self,
        path: ContainerPath,
        link_data: Vec<u8>,
    ) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError>;

    /// Return bridge that matches the given [`ContainerPath`].
    ///
    fn find_bridge(
        &self,
        path: ContainerPath,
    ) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError>;

    /// Retrieve Containers that match given [`ContainerPath`]s.
    ///
    /// If `include_subdirs` is `true`, then the [`ContainerPath`]s are treated as Path prefixes
    /// and not absolute paths.
    ///
    fn find_containers(
        &self,
        paths: Vec<ContainerPath>,
        include_subdirs: bool,
    ) -> Result<Vec<Arc<Mutex<dyn ContainerManifest>>>, CatlibError>;

    /// Retrieve Forest's metadata
    ///
    fn data(&mut self) -> Result<Vec<u8>, CatlibError>;

    /// Retrieve Forest's UUID
    ///
    fn uuid(&self) -> Uuid;

    /// Retrieve Forest's owner identity
    ///
    fn owner(&self) -> Identity;

    /// Retrieve Forests's signers
    ///
    fn signers(&mut self) -> Result<Signers, CatlibError>;
}

/// `ContainerManifest` trait is an API providing methods needed to manipulate container's
/// configuration state. It should be implemented by Cat-Lib instance and should be
/// treated as a kind of a proxy layer between Wildland Core and the external
/// persistent data storage instance (for e.g. database).
///
#[cfg_attr(test, mockall::automock)]
pub trait ContainerManifest: std::fmt::Debug {
    /// Lists the storages that the given container use in order to keep the data.
    ///
    fn get_storages(&mut self) -> Result<Vec<Arc<Mutex<dyn StorageManifest>>>, CatlibError>;

    /// This operation adds a given Storage Backend to the container.
    /// The procedure involves starting the data sync mechanism between the new storage
    /// and the other present storages.
    ///
    /// Container can have multiple storages. Given container should has exact copies
    /// of the data on every storage.
    ///
    fn add_storage(
        &mut self,
        storage_template: &StorageTemplate,
    ) -> Result<Arc<Mutex<dyn StorageManifest>>, CatlibError>;

    /// Returns a printable description of the given container.
    ///
    fn stringify(&self) -> String;

    /// Deletes all paths that the given container contains.
    /// In result the container is considered deleted afterwards.
    /// Container should be treated as a "shared pointer" - once the
    /// last path is deleted the container should be moved to
    /// some sort of a "trash bin".
    ///
    fn delete(&mut self) -> Result<(), CatlibError>;

    /// Return [`Forest`] that contains the [`Container`].
    ///
    fn forest(&self) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>>;

    /// Returns the unique ID of the container.
    ///
    fn uuid(&self) -> Uuid;

    /// Returns true if path was actually added, false otherwise.
    ///
    fn add_path(&mut self, path: String) -> Result<bool, CatlibError>;

    /// Removes the given path. Returns true if the path was actually deleted,
    /// false if the path was not present within the container.
    ///
    fn delete_path(&mut self, path: String) -> Result<bool, CatlibError>;

    /// Lists all the paths from the given container.
    ///
    fn get_paths(&mut self) -> Result<Vec<ContainerPath>, CatlibError>;

    /// User provided name of the container.
    ///
    fn name(&mut self) -> Result<String, CatlibError>;

    /// Sets the user provided name for the container.
    /// This operation involves updating at least the local storage.
    ///
    fn set_name(&mut self, new_name: String) -> Result<(), CatlibError>;
}

#[cfg_attr(test, mockall::automock)]
pub trait StorageManifest: std::fmt::Debug {
    /// Return [`Container`] that contains the [`Storage`].
    fn container(&self) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>>;

    /// Update Storage data
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<()>;

    /// Delete Storage
    fn delete(&mut self) -> CatlibResult<bool>;

    /// Retrieve Storage data
    fn data(&mut self) -> CatlibResult<Vec<u8>>;

    /// Retrieve Storage UUID
    fn uuid(&self) -> Uuid;
}

pub trait BridgeManifest: std::fmt::Debug {
    /// Return [`Forest`] that contains the [`Bridge`].
    fn forest(&self) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>>;

    /// Update Bridge link data
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<()>;

    /// Delete Bridge
    fn delete(&mut self) -> CatlibResult<bool>;

    /// Retrieve Bridge path
    fn path(&mut self) -> CatlibResult<ContainerPath>;
}
