//
// Wildland Project
//
// Copyright © 2022 Golem Foundation,
//               Michał Kluczek <michal@wildland.io>
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

use super::{CatlibResult, Identity};
use std::collections::HashSet;

pub(crate) mod common;

pub type Signers = HashSet<Identity>;
pub type ContainerPath = String;
pub type ContainerPaths = HashSet<ContainerPath>;

pub(crate) trait Forest {
    fn uuid(&self) -> String;
    fn owner(&self) -> Identity;
    fn signers(&self) -> Signers;
    fn data(&self) -> Vec<u8>;
    fn add_signer(&mut self, signer: Identity) -> CatlibResult<bool>;
    fn del_signer(&mut self, signer: Identity) -> CatlibResult<bool>;
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<()>;
    fn remove(&mut self) -> CatlibResult<bool>;
    fn create_container(&self) -> CatlibResult<crate::container::Container>;
    fn create_bridge(
        &self,
        path: ContainerPath,
        link_data: Vec<u8>,
    ) -> CatlibResult<crate::bridge::Bridge>;
}

pub(crate) trait Container {
    fn uuid(&self) -> String;
    fn forest(&self) -> CatlibResult<crate::forest::Forest>;
    fn paths(&self) -> ContainerPaths;
    fn add_path(&mut self, path: ContainerPath) -> CatlibResult<crate::container::Container>;
    fn del_path(&mut self, path: ContainerPath) -> CatlibResult<crate::container::Container>;
    fn storages(&self) -> CatlibResult<Vec<crate::storage::Storage>>;
    fn create_storage(
        &self,
        template_uuid: Option<String>,
        data: Vec<u8>,
    ) -> CatlibResult<crate::storage::Storage>;
}

pub(crate) trait Storage {
    fn uuid(&self) -> String;
    fn template_uuid(&self) -> Option<String>;
    fn container(&self) -> CatlibResult<crate::container::Container>;
    fn data(&self) -> Vec<u8>;
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<crate::storage::Storage>;
}

pub(crate) trait Bridge {
    fn uuid(&self) -> String;
    fn path(&self) -> ContainerPath;
    fn forest(&self) -> CatlibResult<crate::forest::Forest>;
    fn link(&self) -> Vec<u8>;
    fn update(&mut self, data: Vec<u8>) -> CatlibResult<crate::bridge::Bridge>;
}
