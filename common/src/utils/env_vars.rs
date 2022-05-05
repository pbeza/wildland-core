use std::env;

use crate::error::CorexCommonError;
use crate::error::CorexCommonError::EnvVarNotFountError;

pub fn load(key: &str) -> Result<String, CorexCommonError> {
    env::var(key).map_err(|_| EnvVarNotFountError(key.into()))
}
