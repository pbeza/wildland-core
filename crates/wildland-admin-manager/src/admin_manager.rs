use crate::api::user::UserApi;
use std::rc::Rc;
use wildland_corex::LocalSecureStorage;

#[derive(Clone, Debug)]
pub struct AdminManager {
    lss: Rc<dyn LocalSecureStorage>,
    user_api: UserApi,
}

impl AdminManager {
    pub fn new(lss: Rc<dyn LocalSecureStorage>) -> Self {
        Self {
            lss,
            user_api: UserApi,
        }
    }

    pub fn user_api(&self) -> &UserApi {
        &self.user_api
    }
}
