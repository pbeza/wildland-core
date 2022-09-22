use crate::api::{
    foundation_storage::{FoundationStorageApi, FoundationStorageApiConfiguration},
    user::UserApi,
};

/// Structure aggregating and exposing public API of CargoLib library.
#[derive(Clone)]
pub struct CargoLib {
    user_api: UserApi,
    foundation_storage_api: FoundationStorageApi,
}

impl CargoLib {
    pub fn new(user_api: UserApi, fsa_config: FoundationStorageApiConfiguration) -> Self {
        Self {
            user_api,
            foundation_storage_api: FoundationStorageApi::new(fsa_config),
        }
    }

    /// Returns structure aggregating API for user management
    #[tracing::instrument(skip(self))]
    pub fn user_api(&self) -> UserApi {
        self.user_api.clone()
    }

    /// Returns structure aggregating API for Foundation Storage management
    #[tracing::instrument(skip(self))]
    pub fn foundation_storage_api(&self) -> FoundationStorageApi {
        self.foundation_storage_api.clone()
    }
}
