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

mod path_resolver;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub use path_resolver::*;
use thiserror::Error;
use uuid::Uuid;

use crate::{ContainerManifest, ContainerPath, ContainerPaths};

#[derive(Debug, Error)]
pub enum ContainerManagerError {
    #[error("The given container has been already mounted")]
    AlreadyMounted,
    #[error("Generic mounting error {0}")]
    MountingError(String),
    #[error("The given container is not mounted")]
    ContainerNotMounted,
}

#[derive(Default)]
pub struct ContainerManager {
    mounted_containers: HashMap<Uuid, (Arc<Mutex<dyn ContainerManifest>>, ContainerPaths)>,
}

impl ContainerManager {
    pub fn mount(
        &mut self,
        container: &Arc<Mutex<dyn ContainerManifest>>,
    ) -> Result<(), ContainerManagerError> {
        let container_uuid = container.lock().expect("Poisoned Mutex").uuid();
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.mounted_containers.entry(container_uuid)
        {
            let container_paths = container
                .lock()
                .expect("Poisoned Mutex")
                .get_paths()
                .map_err(|e| ContainerManagerError::MountingError(format!("{e}")))?;
            e.insert((
                container.clone(),
                HashSet::from_iter(container_paths.into_iter()),
            ));
            Ok(())
        } else {
            Err(ContainerManagerError::AlreadyMounted)
        }
    }

    pub fn unmount(
        &mut self,
        container: &Arc<Mutex<dyn ContainerManifest>>,
    ) -> Result<(), ContainerManagerError> {
        let container_uuid = container.lock().expect("Poisoned Mutex").uuid();
        self.mounted_containers
            .remove(&container_uuid)
            .ok_or(ContainerManagerError::ContainerNotMounted)
            .map(|_| ())
    }

    pub fn mounted_containers_claiming_path(
        &self,
        path: ContainerPath,
    ) -> Vec<Arc<Mutex<dyn ContainerManifest>>> {
        self.mounted_containers
            .values()
            .filter_map(|(container, paths)| {
                if paths.contains(&path) {
                    Some(container)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }
}

impl PathResolver for ContainerManager {
    fn resolve(&self, _path: &Path) -> Vec<ResolvedPath> {
        todo!() // TODO WILX-353 implement when ContainerManager is filled with information about mounted containers
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

    use uuid::Uuid;

    use crate::catlib_service::entities::MockContainerManifest;
    use crate::{ContainerManager, ContainerManagerError, ContainerManifest};

    #[test]
    fn mount_container() {
        let mut container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".to_owned()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        container_manager.mount(&container1).unwrap();
    }

    #[test]
    fn mount_mounted_container_returns_error() {
        let mut container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".to_owned()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        container_manager.mount(&container1).unwrap();
        assert!(matches!(
            container_manager.mount(&container1),
            Err(ContainerManagerError::AlreadyMounted)
        ));
    }

    #[test]
    fn unmount_container() {
        let mut container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".to_owned()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        container_manager.mount(&container1).unwrap();
        container_manager.unmount(&container1).unwrap();
    }

    #[test]
    fn unmount_not_mounted_container_returns_error() {
        let mut container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".to_owned()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        assert!(matches!(
            container_manager.unmount(&container1),
            Err(ContainerManagerError::ContainerNotMounted)
        ));
    }

    #[test]
    fn get_containers_with_claimed_path() {
        let mut container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".to_owned(), "/some/path2".to_owned()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        let mut container2 = MockContainerManifest::new();
        container2
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".to_owned(), "/some/path3".to_owned()]));
        container2
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap());
        let container2 = Arc::new(Mutex::new(container2)) as Arc<Mutex<dyn ContainerManifest>>;
        container_manager.mount(&container1).unwrap();
        container_manager.mount(&container2).unwrap();

        let containers_claiming_path1 =
            container_manager.mounted_containers_claiming_path("/some/path1".to_owned());
        assert!(containers_claiming_path1
            .iter()
            .any(|container| container.lock().unwrap().uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap()));
        assert!(containers_claiming_path1
            .iter()
            .any(|container| container.lock().unwrap().uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap()));

        let containers_claiming_path2 =
            container_manager.mounted_containers_claiming_path("/some/path2".to_owned());
        assert!(containers_claiming_path2
            .iter()
            .any(|container| container.lock().unwrap().uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap()));
        assert!(!containers_claiming_path2.iter().any(|container| container
            .lock()
            .unwrap()
            .uuid()
            == Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap()));

        let containers_claiming_path3 =
            container_manager.mounted_containers_claiming_path("/some/path3".to_owned());
        assert!(!containers_claiming_path3.iter().any(|container| container
            .lock()
            .unwrap()
            .uuid()
            == Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap()));
        assert!(containers_claiming_path3
            .iter()
            .any(|container| container.lock().unwrap().uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap()));
    }
}
