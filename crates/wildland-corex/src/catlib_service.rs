use std::collections::HashSet;

use serde::Serialize;
use wildland_catlib::{CatLib, CatlibError};

use crate::WildlandIdentity;

#[derive(Clone)]
pub struct CatLibService {
    catlib: CatLib,
}

impl CatLibService {
    #[tracing::instrument(level = "debug")]
    pub fn new() -> Self {
        Self {
            catlib: CatLib::default(),
        }
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn add_forest(
        &self,
        forest_identity: &WildlandIdentity,
        this_device_identity: &WildlandIdentity,
        data: impl Serialize,
    ) -> Result<(), CatlibError> {
        self.catlib
            .create_forest(
                forest_identity.get_public_key().into(),
                HashSet::from([this_device_identity.get_public_key().into()]),
                serde_json::to_vec(&data).map_err(|e| CatlibError::Generic(e.to_string()))?,
            )
            .map(|_| ())
    }
}
