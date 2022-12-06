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

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::CatlibResult;
use std::collections::HashSet;

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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ForestData {
    pub uuid: Uuid,
    pub signers: Signers,
    pub owner: Identity,
    pub data: Vec<u8>,
}

pub trait ForestClone {
    fn clone_box(&self) -> Box<dyn Forest>;
}

impl<T> ForestClone for T
where
    T: 'static + Forest + Clone,
{
    fn clone_box(&self) -> Box<dyn Forest> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Forest> {
    fn clone(&self) -> Box<dyn Forest> {
        self.clone_box()
    }
}

pub trait Forest: AsRef<ForestData> + std::fmt::Debug + ForestClone {
    /// Add manifest Signer
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// - If the signer did not previously exist, `true` is returned.
    /// - If the signer already exists, `false` is returned.
    fn add_signer(&mut self, signer: Identity) -> CatlibResult<bool>;

    /// Delete manifest Signer
    ///
    /// Returns whether the value was already present. That is:
    ///
    /// - If the signer did not previously exist, `false` is returned.
    /// - If the signer existed in the set, `true` is returned.
    fn del_signer(&mut self, signer: Identity) -> CatlibResult<bool>;

    /// Return list of Forest Containers
    fn containers(&self) -> CatlibResult<Vec<Box<dyn Container>>>;

    /// Set Forest arbitrary data
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<&mut dyn Forest>;

    /// Delete Forest
    ///
    /// **WARN: The underlying objects are not removed recursively**
    fn delete(&mut self) -> CatlibResult<bool>;

    /// Create an empty container, bound to the Forest.
    ///
    /// To set container paths, use [`Container::add_path`]
    fn create_container(&self, name: String) -> CatlibResult<Box<dyn Container>>;

    /// Create a Bridge object with arbitrary link data to another Forest.
    ///
    /// The aforementioned link data will be defined by the D/FS module.
    fn create_bridge(
        &self,
        path: ContainerPath,
        link_data: Vec<u8>,
    ) -> CatlibResult<Box<dyn Bridge>>;

    /// Return bridge that matches the given [`ContainerPath`].
    fn find_bridge(&self, path: ContainerPath) -> CatlibResult<Box<dyn Bridge>>;

    /// Retrieve Containers that match given [`ContainerPath`]s.
    ///
    /// If `include_subdirs` is `true`, then the [`ContainerPath`]s are treated as Path prefixes
    /// and not absolute paths.
    fn find_containers(
        &self,
        paths: Vec<ContainerPath>,
        include_subdirs: bool,
    ) -> CatlibResult<Vec<Box<dyn Container>>>;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ContainerData {
    pub uuid: Uuid,
    pub forest_uuid: Uuid,
    pub name: String,
    pub paths: ContainerPaths,
}

pub trait Container: AsRef<ContainerData> + std::fmt::Debug {
    /// Return [`Forest`] that contains the [`Container`].
    fn forest(&self) -> CatlibResult<Box<dyn Forest>>;

    /// Add a path to the Container.
    fn add_path(&mut self, path: ContainerPath) -> CatlibResult<bool>;

    /// Delete a path from the Container.
    fn del_path(&mut self, path: ContainerPath) -> CatlibResult<bool>;

    /// Return list of Forest [`Storage`]s.
    fn storages(&self) -> CatlibResult<Vec<Box<dyn Storage>>>;

    /// Create a [`Storage`], bound to the [`Container`].
    ///
    /// `template_uuid` is an arbitrary, optional, [`String`] that is later used to find
    /// [`Container`]s and [`Storage`]s using [`crate::catlib_service::interface::CatLib::find_storages_with_template()`] and
    /// [`crate::catlib_service::interface::CatLib::find_containers_with_template`].
    ///
    /// `data` represents arbitrary data that is defined and used by the DF/S module.
    fn create_storage(
        &self,
        template_uuid: Option<Uuid>,
        data: Vec<u8>,
    ) -> CatlibResult<Box<dyn Storage>>;

    /// Sets the container's name
    fn set_name(&mut self, new_name: String) -> CatlibResult<()>;

    /// Delete Container
    ///
    /// **WARN: The underlying objects are not removed recursively**
    fn delete(&mut self) -> CatlibResult<bool>;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StorageData {
    pub uuid: Uuid,
    pub container_uuid: Uuid,
    pub template_uuid: Option<Uuid>,
    pub data: Vec<u8>,
}

pub trait Storage: AsRef<StorageData> + std::fmt::Debug {
    /// Return [`Container`] that contains the [`Storage`].
    fn container(&self) -> CatlibResult<Box<dyn Container>>;

    /// Update Storage data
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<&mut dyn Storage>;

    /// Delete Storage
    fn delete(&mut self) -> CatlibResult<bool>;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BridgeData {
    pub uuid: Uuid,
    pub forest_uuid: Uuid,
    pub path: ContainerPath,
    pub link: Vec<u8>,
}

pub trait Bridge: AsRef<BridgeData> + std::fmt::Debug {
    /// Return [`Forest`] that contains the [`Bridge`].
    fn forest(&self) -> CatlibResult<Box<dyn Forest>>;

    /// Update Bridge link data
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<&mut dyn Bridge>;

    /// Delete Bridge
    fn delete(&mut self) -> CatlibResult<bool>;
}
