use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::catlib_service::error::CatlibError;
use crate::{
    ContainerManifest,
    CoreXError,
    ErrContext,
    Storage,
    StorageManifest,
    StorageTemplate,
    StorageTemplateError,
    TemplateContext,
};

#[derive(Clone, Debug)]
pub struct Container {
    container_manifest: Arc<Mutex<dyn ContainerManifest>>,
}

impl Container {
    pub fn new(
        container_manifest: Arc<Mutex<dyn ContainerManifest>>,
        template_uuid: Uuid,
        storage_template: &StorageTemplate,
    ) -> Result<Self, CoreXError> {
        let mut container = Self {
            container_manifest: container_manifest.clone(),
        };

        match container.add_storage(
            template_uuid,
            container.render_template(storage_template).map_err(|e| {
                CoreXError::Generic(format!(
                    "Template render error while creating a container: {e}"
                ))
            })?,
        ) {
            Ok(_) => Ok(container),
            Err(err) => {
                let result: Result<_, CoreXError> = container_manifest
                    .lock()
                    .expect("Poisoned Mutex")
                    .remove()
                    .context(
                        "Error while reverting added container after unsuccessful storage addition",
                    );
                result?;
                Err(CoreXError::CatlibErr(
                    "Storage could not be added while creating Container".into(),
                    err,
                ))
            }
        }
    }

    pub fn from_container_manifest(container_manifest: Arc<Mutex<dyn ContainerManifest>>) -> Self {
        Self { container_manifest }
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`crate::catlib_service::error::CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```
    /// # use wildland_catlib::RedisCatLib;
    /// # use wildland_corex::interface::CatLib;
    /// # use std::collections::{HashSet, HashMap};
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::StorageTemplate;
    /// # use wildland_corex::Forest;
    /// # use uuid::Uuid;
    ///
    /// let catlib = RedisCatLib::default();
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
    /// let container = forest.create_container("container name2".to_owned(), Uuid::new_v4(), &storage_template, path).unwrap();
    /// container.add_path("/bar/baz2".into()).unwrap();
    /// ```
    pub fn add_path(&self, path: String) -> Result<bool, CatlibError> {
        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .add_path(path.into())
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`crate::catlib_service::error::CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use wildland_catlib::RedisCatLib;
    /// # use wildland_corex::interface::CatLib;
    /// # use std::collections::{HashSet, HashMap};
    /// # use wildland_corex::entities::Identity;
    /// # use wildland_corex::StorageTemplate;
    /// # use wildland_corex::Forest;
    /// # use uuid::Uuid;
    /// let catlib = RedisCatLib::default();
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
    /// let container = forest.create_container("container name2".to_owned(), Uuid::new_v4(), &storage_template, path).unwrap();
    /// container.delete_path("/baz/qux1".into()).unwrap();
    /// ```
    ///
    pub fn delete_path(&self, path: String) -> Result<bool, CatlibError> {
        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .delete_path(path.into())
    }

    /// Returns the current collection of paths claimed by the given container.
    ///
    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`crate::catlib_service::error::CatlibResult`] upon failure to save to the database.
    ///
    pub fn get_paths(&self) -> Result<Vec<String>, CatlibError> {
        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .get_paths()
            .map(|paths| {
                paths
                    .into_iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect()
            })
    }

    /// ## Errors
    ///
    /// - Returns [`CatlibError::Generic`] if there was a problem with rendering storage.
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Container`] was found.
    ///
    pub fn add_storage(
        &mut self,
        template_uuid: Uuid,
        storage: Storage,
    ) -> Result<Arc<Mutex<dyn StorageManifest>>, CatlibError> {
        let serialized_storage = serde_json::to_vec(&storage).map_err(|e| {
            CatlibError::Generic(format!("Could not serialize storage template: {e}"))
        })?;

        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .add_storage(template_uuid, serialized_storage)
    }

    fn render_template(
        &self,
        storage_template: &StorageTemplate,
    ) -> Result<Storage, StorageTemplateError> {
        let (container_name, container_uuid, container_paths) = {
            let mut container_lock = self.container_manifest.lock().expect("Poisoned Mutex");
            (
                container_lock
                    .name()
                    .context("Could not retrieve container's name")?,
                container_lock.uuid(),
                container_lock
                    .get_paths()
                    .context("Could not retrieve container's paths")?,
            )
        };
        let template_context = TemplateContext {
            container_name,
            owner: self
                .container_manifest
                .lock()
                .expect("Poisoned Mutex")
                .owner()
                .context("Could not retrieve container's owner")?
                .encode(),
            access_mode: crate::StorageAccessMode::ReadWrite,
            container_uuid,
            paths: container_paths,
        };
        storage_template.render(template_context)
    }

    /// ## Errors
    ///
    /// Returns [`CatlibError::NoRecordsFound`] if Forest has no [`crate::Storage`].
    pub fn get_storages(&mut self) -> Result<Vec<Arc<Mutex<dyn StorageManifest>>>, CatlibError> {
        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .get_storages()
    }

    /// Updates a tet name of the given container.
    ///
    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`crate::catlib_service::error::CatlibResult`] upon failure to save to the database.
    pub fn set_name(&self, new_name: String) -> Result<(), CatlibError> {
        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .set_name(new_name)
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`crate::catlib_service::error::CatlibResult`] upon failure to save to the database.
    pub fn remove(&self) -> Result<(), CatlibError> {
        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .remove()
    }

    /// Get the container's name
    ///
    pub fn name(&self) -> Result<String, CatlibError> {
        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .name()
    }

    /// Get the container's uuid
    ///
    pub fn uuid(&self) -> Uuid {
        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .uuid()
    }

    /// Returns a string representation of the Container object.
    ///
    pub fn stringify(&self) -> String {
        self.container_manifest
            .lock()
            .expect("Poisoned Mutex")
            .stringify()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use uuid::Uuid;

    use crate::*;

    fn make_container(container_uuid: Uuid, path: ContainerPath) -> Container {
        let mut container_manifest = MockContainerManifest::new();
        container_manifest.expect_add_path().returning(|_| Ok(true));
        container_manifest
            .expect_add_storage()
            .returning(|_, _| Ok(Arc::new(Mutex::new(MockStorageManifest::new()))));
        container_manifest
            .expect_delete_path()
            .returning(|_| Ok(true));
        container_manifest
            .expect_forest()
            .returning(move || Ok(Arc::new(Mutex::new(MockForestManifest::new()))));
        container_manifest
            .expect_get_paths()
            .returning(move || Ok(vec![path.clone()]));
        container_manifest
            .expect_get_storages()
            .returning(|| Ok(vec![Arc::new(Mutex::new(MockStorageManifest::new()))]));
        container_manifest
            .expect_name()
            .returning(|| Ok("container name".to_owned()));
        container_manifest.expect_remove().returning(|| Ok(()));
        container_manifest.expect_set_name().returning(|_| Ok(()));
        container_manifest
            .expect_uuid()
            .returning(move || container_uuid);
        container_manifest
            .expect_owner()
            .returning(move || Ok(Identity([1; 32])));
        let storage_template = StorageTemplate::try_new(
            "FoundationStorage",
            &HashMap::from([
                (
                    "field1".to_owned(),
                    "Some value with container name: {{ CONTAINER_NAME }}".to_owned(),
                ),
                (
                    "parameter in key: {{ OWNER }}".to_owned(),
                    "enum: {{ ACCESS_MODE }}".to_owned(),
                ),
                ("uuid".to_owned(), "{{ CONTAINER_UUID }}".to_owned()),
                ("paths".to_owned(), "{{ PATHS }}".to_owned()),
            ]),
        )
        .unwrap();
        Container::new(
            Arc::new(Mutex::new(container_manifest)),
            Uuid::new_v4(),
            &storage_template,
        )
        .unwrap()
    }

    #[test]
    fn new_container_should_has_at_least_one_storage_and_path() {
        let container_uuid = Uuid::new_v4();
        let container = make_container(container_uuid, "/some/path".into());
        let paths = container.get_paths().unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths.contains(&"/some/path".to_string()));
    }
}
