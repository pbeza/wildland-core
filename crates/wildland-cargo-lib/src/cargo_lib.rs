use crate::api::user::UserApi;

#[derive(Default)]
pub struct CargoLib {
    user_api: UserApi,
}
impl CargoLib {
    pub fn user_api(&self) -> &UserApi {
        &self.user_api
    }
}
