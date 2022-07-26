use crate::api::user::UserApi;
use std::rc::Rc;
use wildland_corex::LocalSecureStorage;

#[derive(Clone, Debug)]
pub struct CargoLib {
    _lss: Rc<dyn LocalSecureStorage>,
    user_api: UserApi,
}

impl CargoLib {
    pub fn new(lss: Rc<dyn LocalSecureStorage>) -> Self {
        Self {
            _lss: lss,
            user_api: UserApi,
        }
    }

    pub fn user_api(&self) -> &UserApi {
        &self.user_api
    }
}
