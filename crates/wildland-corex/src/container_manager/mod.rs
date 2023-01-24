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

use crate::{ContainerManifest, ContainerPath, ContainerPaths, Storage};

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
            e.insert((container.clone(), container_paths));
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
    fn resolve(&self, input_path: &Path) -> Result<HashSet<ResolvedPath>, PathResolutionError> {
        let mut physical_paths = Vec::new();
        let mut virtual_paths = HashSet::new();

        for (_uuid, (container_manifest, paths)) in self.mounted_containers.iter() {
            // only first path, which is considered as a primary, is exposed in a filesystem
            if let Some(path_str) = paths.iter().next() {
                match input_path.strip_prefix(path_str) {
                    Ok(stripped_path) => {
                        let mut path_within_storage = PathBuf::from("/");
                        path_within_storage.push(stripped_path);

                        let mut container_manifest =
                            container_manifest.lock().expect("Poisoned mutex");

                        let storages = container_manifest
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
                            storages_id: container_manifest.uuid(),
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
    use crate::{ContainerManager, ContainerManagerError, ContainerManifest};

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
            ContainerManager,
            ContainerManifest,
            MockContainerManifest,
            MockStorageManifest,
            PathResolver,
            Storage,
        };

        fn create_container_with_storage(
            id: u128,
            claimed_paths: Vec<String>,
        ) -> (MockContainerManifest, Storage) {
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

            (container, storage)
        }

        #[test_case(vec!["/".to_owned()], PathBuf::from("/"), Some(PathBuf::from("/")), Some(PathBuf::from("/")); "claimed root")]
        #[test_case(vec!["/path".to_owned()], PathBuf::from("/path"), Some(PathBuf::from("/")), Some(PathBuf::from("/path")); "resolved root within storage")]
        #[test_case(vec!["/path/".to_owned()], PathBuf::from("/path"), Some(PathBuf::from("/")), Some(PathBuf::from("/path")); "slash at the end of claimed path")]
        #[test_case(vec!["/path".to_owned()], PathBuf::from("/path/"), Some(PathBuf::from("/")), Some(PathBuf::from("/path")); "slash at the end of input")]
        #[test_case(vec!["/books".to_owned()], PathBuf::from("/books/fantasy"), Some(PathBuf::from("/fantasy")), None; "one nested component")]
        #[test_case(vec!["/books".to_owned(), "/some/other/path".to_owned()], PathBuf::from("/books/fantasy"), Some(PathBuf::from("/fantasy")), None; "more than one claimed path")]
        #[test_case(vec!["/books/fantasy".to_owned()], PathBuf::from("/books/fantasy/a/lot/of/other/dirs"), Some(PathBuf::from("/a/lot/of/other/dirs")), None; "many nested components")]
        #[test_case(vec!["/books/fantasy".to_owned()], PathBuf::from("/books/"), None, Some(PathBuf::from("/books/fantasy")); "virtual path extended with one component")]
        #[test_case(vec!["/books/fantasy".to_owned()], PathBuf::from("/books"), None, Some(PathBuf::from("/books/fantasy")); "virtual path extended with one component (no slash at input's end)")]
        #[test_case(vec!["/books/fantasy/lord/of/the/rings".to_owned()], PathBuf::from("/books/fantasy"), None, Some(PathBuf::from("/books/fantasy/lord/of/the/rings/")); "virtual path extended with many component")]
        fn test_resolve_with_one_container(
            claimed_paths: Vec<String>,
            resolve_arg_path: PathBuf,
            expected_path_within_storage: Option<PathBuf>,
            expected_virtual_path: Option<PathBuf>,
        ) {
            let mut container_manager = ContainerManager::default();

            let (container1, storage) = create_container_with_storage(1, claimed_paths);

            let container1 = Arc::new(Mutex::new(container1)) as Arc<Mutex<dyn ContainerManifest>>;
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

        #[test_case(PathBuf::from("/"), vec!["/".to_owned()], vec!["/".to_owned()], vec![PathBuf::from("/")], vec![PathBuf::from("/")], vec![PathBuf::from("/")]; "both claiming root")]
        #[test_case(PathBuf::from("/a/b/c"), vec!["/a".to_owned()], vec!["/a/b".to_owned()], vec![PathBuf::from("/b/c")], vec![PathBuf::from("/c")], vec![]; "physical paths from two containers")]
        fn test_resolve_with_two_containers(
            resolve_arg_path: PathBuf,
            claimed_paths_1: Vec<String>,
            claimed_paths_2: Vec<String>,
            expected_paths_within_storage_1: Vec<PathBuf>,
            expected_paths_within_storage_2: Vec<PathBuf>,
            expected_virtual_paths: Vec<PathBuf>,
        ) {
            let mut container_manager = ContainerManager::default();

            let (container_1, storage_1) = create_container_with_storage(1, claimed_paths_1);

            let container_1 =
                Arc::new(Mutex::new(container_1)) as Arc<Mutex<dyn ContainerManifest>>;
            container_manager.mount(&container_1).unwrap();

            let (container_2, storage_2) = create_container_with_storage(2, claimed_paths_2);

            let container_2 =
                Arc::new(Mutex::new(container_2)) as Arc<Mutex<dyn ContainerManifest>>;
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
