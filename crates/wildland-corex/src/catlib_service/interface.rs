use std::sync::{Arc, Mutex};

use uuid::Uuid;

use super::entities::{ContainerManifest, ForestManifest, Identity, Signers, StorageManifest};
use super::error::CatlibResult;

#[cfg_attr(test, mockall::automock)]
pub trait CatLib {
    /// Create new Forest object.
    ///
    /// `owner` and `signers` are cryptographical objects that are used by the Core module to
    /// verify the cryptographical integrity of the manifests.
    ///
    /// `data` is an arbitrary data object that can be used to synchronize Forest state between
    /// devices.
    fn create_forest(
        &self,
        owner: Identity,
        signers: Signers,
        data: Vec<u8>,
    ) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>>;

    /// Return [`crate::Forest`] object by Forest UUID.
    fn get_forest(&self, uuid: &Uuid) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>>;

    /// Return [`crate::Forest`] owned by specified `owner`.
    fn find_forest(&self, owner: &Identity) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>>;

    /// Return [`crate::Container`] object by Container UUID.
    fn get_container(&self, uuid: &Uuid) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>>;

    /// Return [`crate::Storage`]s that were created using given `template_id` UUID.
    fn find_storages_with_template(
        &self,
        template_id: &Uuid,
    ) -> CatlibResult<Vec<Arc<Mutex<dyn StorageManifest>>>>;

    /// Return [`crate::Container`]s that were created using given `template_id` UUID.
    fn find_containers_with_template(
        &self,
        template_id: &Uuid,
    ) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>>;

    /// Save StorageTemplate data in CatLib.
    fn save_storage_template(&self, template_id: &Uuid, value: String) -> CatlibResult<()>;

    /// Fetche every StorageTemplate data from CatLib.
    fn get_storage_templates_data(&self) -> CatlibResult<Vec<String>>;

    /// Check if database backend is available
    fn is_db_alive(&self) -> CatlibResult<bool>;
}
