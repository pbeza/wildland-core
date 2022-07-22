use crate::{CoreXError, CorexResult};
use std::path::PathBuf;
use wildland_local_secure_storage::FileLSS;

pub fn create_file_lss(path: String) -> CorexResult<FileLSS> {
    FileLSS::new(PathBuf::from(path))
        .map_err(CoreXError::from)
}
