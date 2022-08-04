use crate::api::user::UserApi;

#[derive(Clone, Debug)]
pub struct CargoLib {
    user_api: UserApi,
}

impl CargoLib {
    pub fn new(user_api: UserApi) -> Self {
        Self { user_api }
    }

    pub fn user_api(&self) -> UserApi {
        self.user_api.clone()
    }
}
