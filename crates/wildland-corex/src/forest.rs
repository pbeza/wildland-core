use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::catlib_service::error::CatlibError;
use crate::entities::Identity;
use crate::{BridgeManifest, Container, ContainerPath, ForestManifest, StorageTemplate};

#[derive(Debug, Clone)]
pub struct Forest {
    forest_manifest: Arc<Mutex<dyn ForestManifest>>,
}

impl Forest {
    pub fn new(forest_manifest: Arc<Mutex<dyn ForestManifest>>) -> Self {
        Self { forest_manifest }
    }

    pub fn forest_manifest(&self) -> Arc<Mutex<dyn ForestManifest>> {
        self.forest_manifest.clone()
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use wildland_catlib::CatLib;
    /// # use std::collections::{HashSet, HashMap};
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::StorageTemplate;
    /// # use wildland_corex::interface::CatLib as ICatLib;
    /// # use wildland_corex::Forest;
    ///
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  Identity([1; 32]),
    ///                  HashSet::from([Identity([2; 32])]),
    ///                  vec![],
    ///              ).unwrap();
    /// let forest = Forest::new(forest);
    /// let storage_template = StorageTemplate::try_new(
    ///     "FoundationStorage",
    ///     &HashMap::from([
    ///             (
    ///                 "field1".to_owned(),
    ///                 "Some value with container name: {{ CONTAINER_NAME }}".to_owned(),
    ///             ),
    ///             (
    ///                 "parameter in key: {{ OWNER }}".to_owned(),
    ///                 "enum: {{ ACCESS_MODE }}".to_owned(),
    ///             ),
    ///             ("uuid".to_owned(), "{{ CONTAINER_UUID }}".to_owned()),
    ///             ("paths".to_owned(), "{{ PATHS }}".to_owned()),
    ///         ]),
    ///     )
    ///     .unwrap();
    /// let path = "/some/path".into();
    /// let container = forest.create_container("container name1".to_owned(), &storage_template, path).unwrap();
    /// container.add_path("/foo/bar1".into());
    /// container.add_path("/bar/baz1".into());
    /// ```
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_container(
        &self,
        name: String,
        storage_template: &StorageTemplate,
        path: ContainerPath,
    ) -> Result<Container, CatlibError> {
        let container_uuid = Uuid::new_v4();
        let forest_uuid = self.forest_manifest.lock().expect("Poisoned Mutex").uuid();
        let container_manifest = self
            .forest_manifest
            .lock()
            .expect("Poisoned Mutex")
            .create_container(container_uuid, forest_uuid, name, path)?;
        Container::new(container_manifest, storage_template)
    }

    pub fn containers(&self) -> Result<Vec<Container>, CatlibError> {
        self.forest_manifest
            .lock()
            .expect("Poisoned Mutex")
            .containers()
            .map(|containers_vec| {
                containers_vec
                    .into_iter()
                    .map(|container_manifest| {
                        Container::from_container_manifest(container_manifest)
                    })
                    .collect()
            })
    }

    pub fn find_containers(
        &self,
        paths: Vec<String>,
        include_subdirs: bool,
    ) -> Result<Vec<Container>, CatlibError> {
        self.forest_manifest
            .lock()
            .expect("Poisoned Mutex")
            .find_containers(
                paths.into_iter().map(PathBuf::from).collect(),
                include_subdirs,
            )
            .map(|containers_vec| {
                containers_vec
                    .into_iter()
                    .map(|container_manifest| {
                        Container::from_container_manifest(container_manifest)
                    })
                    .collect()
            })
    }

    pub fn create_bridge(
        &self,
        path: String,
        link_data: Vec<u8>,
    ) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError> {
        self.forest_manifest
            .lock()
            .expect("Poisoned Mutex")
            .create_bridge(path, link_data)
    }

    pub fn find_bridge(&self, path: String) -> Result<Arc<Mutex<dyn BridgeManifest>>, CatlibError> {
        self.forest_manifest
            .lock()
            .expect("Poisoned Mutex")
            .find_bridge(path)
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    pub fn add_signer(&mut self, signer: Identity) -> Result<bool, CatlibError> {
        self.forest_manifest
            .lock()
            .expect("Poisoned Mutex")
            .add_signer(signer)
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn delete_signer(&self, signer: Identity) -> Result<bool, CatlibError> {
        self.forest_manifest
            .lock()
            .expect("Poisoned Mutex")
            .delete_signer(signer)
    }

    /// Gets the current collection of the Forest's signers.
    ///
    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to sync to the database.
    pub fn signers(&mut self) -> Result<Vec<Identity>, CatlibError> {
        self.forest_manifest
            .lock()
            .expect("Poisoned Mutex")
            .signers()
            .map(|signers| signers.into_iter().collect())
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove(self) -> Result<bool, CatlibError> {
        self.forest_manifest
            .lock()
            .expect("Poisoned Mutex")
            .remove()
    }

    pub fn uuid(&self) -> Uuid {
        self.forest_manifest.lock().expect("Poisoned Mutex").uuid()
    }

    pub fn owner(&self) -> Identity {
        self.forest_manifest.lock().expect("Poisoned Mutex").owner()
    }
}
