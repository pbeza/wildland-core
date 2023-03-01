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

use std::sync::{Arc, Mutex};

use uuid::Uuid;
use wildland_corex::catlib_service::error::CatlibError;
use wildland_corex::{
    CatlibContainerFilter,
    Container,
    ContainerManager,
    ContainerManagerError,
    StorageManifest,
    StorageTemplate,
};

#[derive(Debug, Clone)]
pub struct CargoContainerFilter {
    inner: CatlibContainerFilter,
}

impl From<CargoContainerFilter> for CatlibContainerFilter {
    fn from(val: CargoContainerFilter) -> CatlibContainerFilter {
        val.inner
    }
}

impl CargoContainerFilter {
    pub fn has_exact_path(path: String) -> Self {
        Self {
            inner: CatlibContainerFilter::HasExactPath(path.into()),
        }
    }

    pub fn has_path_starting_with(path: String) -> Self {
        Self {
            inner: CatlibContainerFilter::HasPathStartingWith(path.into()),
        }
    }

    pub fn or(f1: Self, f2: Self) -> Self {
        Self {
            inner: CatlibContainerFilter::Or(Box::new(f1.into()), Box::new(f2.into())),
        }
    }

    pub fn and(f1: Self, f2: Self) -> Self {
        Self {
            inner: CatlibContainerFilter::And(Box::new(f1.into()), Box::new(f2.into())),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(f: Self) -> Self {
        Self {
            inner: CatlibContainerFilter::Not(Box::new(f.into())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum MountState {
    Mounted,
    Unmounted,
    MountedOrUnmounted,
}

#[derive(Debug, Clone)]
pub struct CargoContainer {
    container_manager: ContainerManager,
    corex_container: Container,
}

impl CargoContainer {
    pub fn new(container_manager: ContainerManager, corex_container: Container) -> Self {
        Self {
            container_manager,
            corex_container,
        }
    }

    /// CargoLib methods

    pub fn mount(&self) -> Result<(), ContainerManagerError> {
        self.container_manager.mount(&self.corex_container)
    }

    pub fn unmount(&self) -> Result<(), ContainerManagerError> {
        self.container_manager.unmount(&self.corex_container)
    }

    pub fn is_mounted(&self) -> bool {
        self.container_manager.is_mounted(&self.corex_container)
    }

    /// Corex methods

    pub fn get_storages(&mut self) -> Result<Vec<Arc<Mutex<dyn StorageManifest>>>, CatlibError> {
        self.corex_container.get_storages()
    }

    pub fn add_storage(
        &mut self,
        storage_template: &StorageTemplate,
    ) -> Result<Arc<Mutex<dyn StorageManifest>>, CatlibError> {
        self.corex_container.add_storage(storage_template)
    }

    pub fn add_path(&self, path: String) -> Result<bool, CatlibError> {
        self.corex_container.add_path(path)
    }

    pub fn delete_path(&self, path: String) -> Result<bool, CatlibError> {
        self.corex_container.delete_path(path)
    }

    pub fn get_paths(&self) -> Result<Vec<String>, CatlibError> {
        self.corex_container.get_paths()
    }

    pub fn set_name(&self, new_name: String) -> Result<(), CatlibError> {
        self.corex_container.set_name(new_name)
    }

    pub fn remove(&self) -> Result<(), CatlibError> {
        self.corex_container.remove()
    }

    pub fn name(&self) -> Result<String, CatlibError> {
        self.corex_container.name()
    }

    pub fn uuid(&self) -> Uuid {
        self.corex_container.uuid()
    }

    pub fn stringify(&self) -> String {
        self.corex_container.stringify()
    }
}
