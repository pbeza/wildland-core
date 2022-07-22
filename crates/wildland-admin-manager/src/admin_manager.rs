use crate::api::user::UserApi;

#[derive(Default)]
pub struct AdminManager {
    user_api: UserApi,
}
impl AdminManager {
    pub fn user_api(&self) -> &UserApi {
        &self.user_api
    }
}
