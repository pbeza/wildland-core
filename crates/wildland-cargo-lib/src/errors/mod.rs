mod retrieval_error;
mod single_variant;
mod user;
pub use retrieval_error::*;
pub use single_variant::*;
pub use user::*;

use crate::api::foundation_storage::FsaError;

impl ExceptionTrait for FsaError {
    fn reason(&self) -> String {
        self.to_string()
    }
}

pub trait ExceptionTrait {
    fn reason(&self) -> String;
}
