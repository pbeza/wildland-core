use crate::api::user::UserApi;

/// Structure aggregating and exposing public API of CargoLib library.
#[derive(Clone)]
pub struct CargoLib {
    user_api: UserApi,
}

impl CargoLib {
    pub fn new(user_api: UserApi) -> Self {
        Self { user_api }
    }

    /// Returns structure aggregating API for user management
    #[tracing::instrument(skip(self))]
    pub fn user_api(&self) -> UserApi {
        self.user_api.clone()
    }
}
