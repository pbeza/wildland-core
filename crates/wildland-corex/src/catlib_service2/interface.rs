use uuid::Uuid;

use super::{
    entities::{Container, Forest, Identity, Signers, Storage},
    error::CatlibResult,
};

pub trait CatLib {
    /// Create new Forest object.
    ///
    /// `owner` and `signers` are cryptographical objects that are used by the Core module to
    /// verify the cryptographical integrity of the manifests.
    ///
    /// `data` is an arbitrary data object that can be used to synchronize Forest state between
    /// devices.
    ///
    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use wildland_catlib::{CatLib, Identity};
    /// # use std::collections::HashSet;
    /// # use crate::wildland_catlib::IForest;
    /// let forest_owner = Identity([1; 32]);
    /// let signer = Identity([2; 32]);
    ///
    /// let catlib = CatLib::default();
    /// let forest = catlib.create_forest(
    ///                  forest_owner,
    ///                  HashSet::from([signer]),
    ///                  vec![],
    ///              ).unwrap();
    /// ```
    fn create_forest(
        &self,
        owner: Identity,
        signers: Signers,
        data: Vec<u8>,
    ) -> CatlibResult<Box<dyn Forest>>;

    /// Return [`Forest`] object by Forest UUID.
    fn get_forest(&self, uuid: Uuid) -> CatlibResult<Box<dyn Forest>>;

    /// Return [`Forest`] owned by specified `owner`.
    ///
    /// **Note: by design each owner may have only one Forest**
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Forest`] was found.
    fn find_forest(&self, owner: Identity) -> CatlibResult<Box<dyn Forest>>;

    /// Return [`Container`] object by Container UUID.
    fn get_container(&self, uuid: Uuid) -> CatlibResult<Box<dyn Container>>;

    /// Return [`Storage`]s that were created using given `template_id` UUID.
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Forest`] was found.
    fn find_storages_with_template(&self, template_id: Uuid)
        -> CatlibResult<Vec<Box<dyn Storage>>>;

    /// Return [`Container`]s that were created using given `template_id` UUID.
    ///
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseEntry`] if more than one [`Forest`] was found.
    fn find_containers_with_template(
        &self,
        template_id: Uuid,
    ) -> CatlibResult<Vec<Box<dyn Container>>>;
}
