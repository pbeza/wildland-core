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

use super::*;
use super::{CatlibResult, Identity};
use std::collections::HashSet;

pub(crate) mod common;

pub type Signers = HashSet<Identity>;
pub type ContainerPath = String;
pub type ContainerPaths = HashSet<ContainerPath>;

pub trait IForest {
    /// Return UUID object identifier
    fn uuid(&self) -> Uuid;

    /// Return Forest owner
    fn owner(&self) -> Identity;

    /// Return list of manifests Signers
    fn signers(&self) -> Signers;

    /// Return Forest arbitrary data
    fn data(&self) -> Vec<u8>;

    /// Add manifest Signer
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// - If the signer did not previously exist, `true` is returned.
    /// - If the signer already exists, `false` is returned.
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn add_signer(&mut self, signer: Identity) -> CatlibResult<bool>;

    /// Delete manifest Signer
    ///
    /// Returns whether the value was already present. That is:
    ///
    /// - If the signer did not previously exist, `false` is returned.
    /// - If the signer existed in the set, `true` is returned.
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn del_signer(&mut self, signer: Identity) -> CatlibResult<bool>;

    /// Return list of Forest Containers
    ///
    /// ## Errors
    ///
    /// Returns [`CatlibError::NoRecordsFound`] if Forest has no [`Container`].
    fn containers(&self) -> CatlibResult<Vec<Container>>;

    /// Set Forest arbitrary data
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<()>;

    /// Delete Forest from the database
    ///
    /// **WARN: The underlying objects are not removed recursively**
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn remove(&mut self) -> CatlibResult<bool>;

    /// Create an empty container, bound to the Forest.
    ///
    /// To set container paths, use [`Container::add_path`]
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use crate::wildland_catlib::*;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let mut container = forest.create_container("container name".to_owned()).unwrap();
    /// container.add_path("/foo/bar".to_string());
    /// container.add_path("/bar/baz".to_string());
    /// ```
    fn create_container(&self, name: String) -> CatlibResult<Container>;

    /// Create a Bridge obect with arbitrary link data to another Forest.
    ///
    /// The aforementioned link data will be defined by the D/FS module.
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use crate::wildland_catlib::*;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// forest.create_bridge("/other/forest".to_string(), vec![]);
    /// ```
    fn create_bridge(&self, path: ContainerPath, link_data: Vec<u8>) -> CatlibResult<Bridge>;

    /// Return bridge that matches the given [`ContainerPath`].
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Bridge`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Bridge`] was found.
    fn find_bridge(&self, path: ContainerPath) -> CatlibResult<Bridge>;

    /// Retrieve Containers that match given [`ContainerPath`]s.
    ///
    /// If `include_subdirs` is `true`, then the [`ContainerPath`]s are treated as Path prefixes
    /// and not absolute paths.
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    ///
    /// ## Example
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use crate::wildland_catlib::*;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  b"owner".to_vec(),
    ///                  HashSet::from([b"signer".to_vec()]),
    ///                  vec![],
    ///              ).unwrap();
    /// let mut container = forest.create_container().unwrap();
    /// container.add_path("/foo/bar".to_string());
    ///
    /// let containers = forest.find_containers(vec!["/foo/bar".to_string()], false).unwrap();
    fn find_containers(
        &self,
        paths: Vec<ContainerPath>,
        include_subdirs: bool,
    ) -> CatlibResult<Vec<Container>>;
}

pub trait IContainer {
    /// Return UUID object identifier
    fn uuid(&self) -> Uuid;

    /// Returns name of the container
    fn name(&self) -> String;

    /// Sets the container's name
    fn set_name(&mut self, new_name: String);

    /// Return [`Forest`] that contains the [`Container`].
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Forest`] was found.
    fn forest(&self) -> CatlibResult<Forest>;

    /// Return [`Container`]'s paths
    fn paths(&self) -> ContainerPaths;

    /// Add a path to the Container.
    ///
    /// Returns self to allow chain method exection.
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use crate::wildland_catlib::*;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let mut container = forest.create_container("container name".to_owned()).unwrap();
    /// container.add_path("/bar/baz2".to_string()).unwrap()
    ///     .add_path("/baz/qux1".to_string()).unwrap()
    ///     .add_path("/baz/qux2".to_string()).unwrap();
    /// ```
    fn add_path(&mut self, path: ContainerPath) -> CatlibResult<Container>;

    /// Delete a path from the Container.
    ///
    /// Returns self to allow chain method exection.
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use crate::wildland_catlib::*;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let mut container = forest.create_container("container name".to_owned()).unwrap();
    /// container.add_path("/bar/baz2".to_string()).unwrap()
    ///     .del_path("/baz/qux1".to_string()).unwrap()
    ///     .del_path("/baz/qux2".to_string()).unwrap();
    /// ```
    fn del_path(&mut self, path: ContainerPath) -> CatlibResult<Container>;

    /// Return list of Forest [`Storage`]s.
    ///
    /// ## Errors
    ///
    /// Returns [`CatlibError::NoRecordsFound`] if Forest has no [`Storage`].
    fn storages(&self) -> CatlibResult<Vec<Storage>>;

    /// Create a [`Storage`], bound to the [`Container`].
    ///
    /// `template_uuid` is an arbitrary, optional, [`String`] that is later used to find
    /// [`Container`]s and [`Storage`]s using [`CatLib::find_storages_with_template`] and
    /// [`CatLib::find_containers_with_template`].
    ///
    /// `data` represents arbitrary data that is defined and used by the DF/S module.
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::HashSet;
    /// # use crate::wildland_catlib::*;
    /// # use uuid::Uuid;
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let mut container = forest.create_container("container name".to_owned()).unwrap();
    /// container.add_path("/foo/bar".to_string());
    /// container.create_storage(Some(Uuid::from_u128(1)), vec![]).unwrap();
    /// ```
    fn create_storage(&self, template_uuid: Option<Uuid>, data: Vec<u8>) -> CatlibResult<Storage>;
}

pub trait IStorage {
    /// Return UUID object identifier
    fn uuid(&self) -> Uuid;

    /// Return Template UUID
    fn template_uuid(&self) -> Option<Uuid>;

    /// Return [`Container`] that contains the [`Storage`].
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Container`] was found.
    fn container(&self) -> CatlibResult<Container>;

    /// Return Storage data
    fn data(&self) -> Vec<u8>;

    /// Update Storage data
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<Storage>;
}

pub trait IBridge {
    /// Return UUID object identifier
    fn uuid(&self) -> Uuid;

    // Returns [`Bridge`]'s path
    fn path(&self) -> ContainerPath;

    /// Return [`Forest`] that contains the [`Bridge`].
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Bridge`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Bridge`] was found.
    fn forest(&self) -> CatlibResult<Forest>;

    /// Return Bridge link data
    fn link(&self) -> Vec<u8>;

    /// Update Bridge link data
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<Bridge>;
}
