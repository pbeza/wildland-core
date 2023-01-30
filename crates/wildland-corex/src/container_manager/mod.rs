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
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub use path_resolver::*;
use thiserror::Error;
use uuid::Uuid;

use crate::{Container, ContainerPath, ContainerPaths, Storage};

#[derive(Debug, Error, Clone)]
#[repr(C)]
pub enum ContainerManagerError {
    #[error("The given container has been already mounted")]
    AlreadyMounted,
    #[error("Generic mounting error {0}")]
    MountingError(String),
    #[error("The given container is not mounted")]
    ContainerNotMounted,
}

type MountedContainerMap = Arc<Mutex<HashMap<Uuid, (Container, ContainerPaths)>>>;

#[derive(Default, Clone)]
pub struct ContainerManager {
    mounted_containers: MountedContainerMap,
}

impl ContainerManager {
    pub fn mount(&self, container: &Container) -> Result<(), ContainerManagerError> {
        let container_uuid = container.uuid();
        if let std::collections::hash_map::Entry::Vacant(e) = self
            .mounted_containers
            .lock()
            .expect("Poisoned mutex!")
            .entry(container_uuid)
        {
            let container_paths = container
                .get_paths()
                .map(|paths| paths.into_iter().map(PathBuf::from).collect())
                .map_err(|e| ContainerManagerError::MountingError(format!("{e}")))?;
            e.insert((container.clone(), container_paths));
            Ok(())
        } else {
            Err(ContainerManagerError::AlreadyMounted)
        }
    }

    pub fn unmount(&self, container: &Container) -> Result<(), ContainerManagerError> {
        let container_uuid = container.uuid();
        self.mounted_containers
            .lock()
            .expect("Poisoned mutex!")
            .remove(&container_uuid)
            .ok_or(ContainerManagerError::ContainerNotMounted)
            .map(|_| ())
    }

    pub fn mounted_containers_claiming_path(&self, path: ContainerPath) -> Vec<Container> {
        self.mounted_containers
            .lock()
            .expect("Poisoned mutex!")
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
    fn resolve(&self, input_path: &Path) -> Result<HashSet<ResolvedPath>, PathResolutionError> {
        let mut physical_paths = Vec::new();
        let mut virtual_paths = HashSet::new();

        for (_uuid, (container, paths)) in self
            .mounted_containers
            .lock()
            .expect("Poisoned mutex!")
            .iter_mut()
        {
            // only first path, which is considered as a primary, is exposed in a filesystem
            if let Some(path_str) = paths.iter().next() {
                match input_path.strip_prefix(path_str) {
                    Ok(stripped_path) => {
                        let mut path_within_storage = PathBuf::from("/");
                        path_within_storage.push(stripped_path);

                        let storages = container
                            .get_storages()?
                            .into_iter()
                            .map(|storage_manifest| {
                                let mut storage_manifest =
                                    storage_manifest.lock().expect("Poisoned mutex");

                                let data = storage_manifest.data()?;
                                serde_json::from_slice::<Storage>(
                                    &data,
                                ).map_err(|e| {
                                    tracing::error!(
                                        "Could nod deserialize Storage Manifest stored in CatLib: {}",
                                        e.to_string(),
                                    );
                                    PathResolutionError::Generic(e.to_string())
                                })
                            })
                            .collect::<Result<Vec<Storage>, PathResolutionError>>()?;

                        if Path::new(path_str) == input_path {
                            virtual_paths
                                .insert(ResolvedPath::VirtualPath(PathBuf::from(path_str)));
                        }

                        let physical_path = ResolvedPath::PathWithStorages {
                            path_within_storage,
                            storages_id: container.uuid(),
                            storages,
                        };

                        physical_paths.push(physical_path);
                    }
                    Err(_err) => {
                        if Path::new(path_str).starts_with(input_path) {
                            virtual_paths
                                .insert(ResolvedPath::VirtualPath(PathBuf::from(path_str)));
                        }
                    }
                }
            }
        }

        Ok(physical_paths
            .into_iter()
            .chain(virtual_paths.into_iter())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

    use uuid::Uuid;

    use crate::catlib_service::entities::MockContainerManifest;
    use crate::{Container, ContainerManager, ContainerManagerError, ContainerManifest};

    mod path_resolver_tests {
        use std::collections::HashSet;
        use std::path::PathBuf;
        use std::sync::{Arc, Mutex};

        use pretty_assertions::assert_eq;
        use serde_json::json;
        use test_case::test_case;
        use uuid::Uuid;

        use crate::ResolvedPath::{self, PathWithStorages};
        use crate::{
            Container,
            ContainerManager,
            ContainerPaths,
            MockContainerManifest,
            MockStorageManifest,
            PathResolver,
            Storage,
        };

        fn create_container_with_storage(
            id: u128,
            claimed_paths: ContainerPaths,
        ) -> (Container, Storage) {
            let storage = Storage::new(None, format!("test backend type {id}"), json!({}));
            let storage_uuid = storage.uuid();

            let mut container = MockContainerManifest::new();
            container
                .expect_uuid()
                .returning(move || Uuid::from_u128(id));
            container
                .expect_get_paths()
                .returning(move || Ok(claimed_paths.clone()));
            container.expect_get_storages().returning(move || {
                let mut storage_manifest = MockStorageManifest::new();
                storage_manifest.expect_data().returning(move || {
                    Ok(serde_json::to_vec(&json!({
                        "uuid": storage_uuid,
                        "backend_type": format!("test backend type {id}"),
                        "data": json!({})
                    }))
                    .unwrap())
                });
                Ok(vec![Arc::new(Mutex::new(storage_manifest))])
            });

            let container = Container::from_container_manifest(Arc::new(Mutex::new(container)));

            (container, storage)
        }

        #[test_case(vec!["/".into()], PathBuf::from("/"), Some(PathBuf::from("/")), Some(PathBuf::from("/")); "claimed root")]
        #[test_case(vec!["/path".into()], PathBuf::from("/path"), Some(PathBuf::from("/")), Some(PathBuf::from("/path")); "resolved root within storage")]
        #[test_case(vec!["/path/".into()], PathBuf::from("/path"), Some(PathBuf::from("/")), Some(PathBuf::from("/path")); "slash at the end of claimed path")]
        #[test_case(vec!["/path".into()], PathBuf::from("/path/"), Some(PathBuf::from("/")), Some(PathBuf::from("/path")); "slash at the end of input")]
        #[test_case(vec!["/books".into()], PathBuf::from("/books/fantasy"), Some(PathBuf::from("/fantasy")), None; "one nested component")]
        #[test_case(vec!["/books".into(), "/some/other/path".into()], PathBuf::from("/books/fantasy"), Some(PathBuf::from("/fantasy")), None; "more than one claimed path")]
        #[test_case(vec!["/books/fantasy".into()], PathBuf::from("/books/fantasy/a/lot/of/other/dirs"), Some(PathBuf::from("/a/lot/of/other/dirs")), None; "many nested components")]
        #[test_case(vec!["/books/fantasy".into()], PathBuf::from("/books/"), None, Some(PathBuf::from("/books/fantasy")); "virtual path extended with one component")]
        #[test_case(vec!["/books/fantasy".into()], PathBuf::from("/books"), None, Some(PathBuf::from("/books/fantasy")); "virtual path extended with one component (no slash at input's end)")]
        #[test_case(vec!["/books/fantasy/lord/of/the/rings".into()], PathBuf::from("/books/fantasy"), None, Some(PathBuf::from("/books/fantasy/lord/of/the/rings/")); "virtual path extended with many component")]
        fn test_resolve_with_one_container(
            claimed_paths: ContainerPaths,
            resolve_arg_path: PathBuf,
            expected_path_within_storage: Option<PathBuf>,
            expected_virtual_path: Option<PathBuf>,
        ) {
            let container_manager = ContainerManager::default();

            let (container1, storage) = create_container_with_storage(1, claimed_paths);

            let container1 = container1;
            container_manager.mount(&container1).unwrap();

            let resolved_paths = container_manager.resolve(&resolve_arg_path).unwrap();
            let expected: HashSet<ResolvedPath> = [
                expected_path_within_storage.map(|path_within_storage| PathWithStorages {
                    path_within_storage,
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage],
                }),
                expected_virtual_path.map(ResolvedPath::VirtualPath),
            ]
            .into_iter()
            .flatten()
            .collect();
            assert_eq!(expected, resolved_paths);
        }

        #[test_case(PathBuf::from("/"), vec!["/".into()], vec!["/".into()], vec![PathBuf::from("/")], vec![PathBuf::from("/")], vec![PathBuf::from("/")]; "both claiming root")]
        #[test_case(PathBuf::from("/a/b/c"), vec!["/a".into()], vec!["/a/b".into()], vec![PathBuf::from("/b/c")], vec![PathBuf::from("/c")], vec![]; "physical paths from two containers")]
        #[test_case(PathBuf::from("/a/b/c"), vec!["/a/b/c".into()], vec!["/a/b".into()], vec![PathBuf::from("/")], vec![PathBuf::from("/c")], vec![PathBuf::from("/a/b/c")]; "physical and virtual paths from two containers")]
        fn test_resolve_with_two_containers(
            resolve_arg_path: PathBuf,
            claimed_paths_1: ContainerPaths,
            claimed_paths_2: ContainerPaths,
            expected_paths_within_storage_1: Vec<PathBuf>,
            expected_paths_within_storage_2: Vec<PathBuf>,
            expected_virtual_paths: Vec<PathBuf>,
        ) {
            let container_manager = ContainerManager::default();

            let (container_1, storage_1) = create_container_with_storage(1, claimed_paths_1);
            container_manager.mount(&container_1).unwrap();

            let (container_2, storage_2) = create_container_with_storage(2, claimed_paths_2);
            container_manager.mount(&container_2).unwrap();

            let resolved_paths = container_manager.resolve(&resolve_arg_path).unwrap();
            let expected: HashSet<ResolvedPath> = expected_paths_within_storage_1
                .into_iter()
                .map(|path_within_storage| PathWithStorages {
                    path_within_storage,
                    storages_id: Uuid::from_u128(1),
                    storages: vec![storage_1.clone()],
                })
                .chain(
                    expected_paths_within_storage_2
                        .into_iter()
                        .map(|path_within_storage| PathWithStorages {
                            path_within_storage,
                            storages_id: Uuid::from_u128(2),
                            storages: vec![storage_2.clone()],
                        }),
                )
                .chain(
                    expected_virtual_paths
                        .into_iter()
                        .map(ResolvedPath::VirtualPath),
                )
                .collect();
            assert_eq!(expected, resolved_paths.into_iter().collect());
        }
    }

    #[test]
    fn mount_container() {
        let container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".into()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        container_manager
            .mount(&Container::from_container_manifest(container1))
            .unwrap();
    }

    #[test]
    fn mount_mounted_container_returns_error() {
        let container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".into()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        container_manager
            .mount(&Container::from_container_manifest(container1.clone()))
            .unwrap();
        assert!(matches!(
            container_manager.mount(&Container::from_container_manifest(container1)),
            Err(ContainerManagerError::AlreadyMounted)
        ));
    }

    #[test]
    fn unmount_container() {
        let container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".into()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        container_manager
            .mount(&Container::from_container_manifest(container1.clone()))
            .unwrap();
        container_manager
            .unmount(&Container::from_container_manifest(container1))
            .unwrap();
    }

    #[test]
    fn unmount_not_mounted_container_returns_error() {
        let container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".into()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        assert!(matches!(
            container_manager.unmount(&Container::from_container_manifest(container1)),
            Err(ContainerManagerError::ContainerNotMounted)
        ));
    }

    #[test]
    fn get_containers_with_claimed_path() {
        let container_manager = ContainerManager::default();
        let mut container1 = MockContainerManifest::new();
        container1
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".into(), "/some/path2".into()]));
        container1
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap());
        let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
        let mut container2 = MockContainerManifest::new();
        container2
            .expect_get_paths()
            .returning(|| Ok(vec!["/some/path1".into(), "/some/path3".into()]));
        container2
            .expect_uuid()
            .returning(|| Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap());
        let container2 = Arc::new(Mutex::new(container2)) as Arc<Mutex<dyn ContainerManifest>>;
        container_manager
            .mount(&Container::from_container_manifest(container1))
            .unwrap();
        container_manager
            .mount(&Container::from_container_manifest(container2))
            .unwrap();

        let containers_claiming_path1 =
            container_manager.mounted_containers_claiming_path("/some/path1".into());
        assert!(containers_claiming_path1
            .iter()
            .any(|container| container.uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap()));
        assert!(containers_claiming_path1
            .iter()
            .any(|container| container.uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap()));

        let containers_claiming_path2 =
            container_manager.mounted_containers_claiming_path("/some/path2".into());
        assert!(containers_claiming_path2
            .iter()
            .any(|container| container.uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap()));
        assert!(!containers_claiming_path2
            .iter()
            .any(|container| container.uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap()));

        let containers_claiming_path3 =
            container_manager.mounted_containers_claiming_path("/some/path3".into());
        assert!(!containers_claiming_path3
            .iter()
            .any(|container| container.uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap()));
        assert!(containers_claiming_path3
            .iter()
            .any(|container| container.uuid()
                == Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap()));
    }
}
