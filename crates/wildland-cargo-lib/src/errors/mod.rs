mod creation_error;
mod retrieval_error;
mod user;

pub use creation_error::*;
pub use retrieval_error::*;
pub use user::*;

pub trait ExceptionTrait {
    fn reason(&self) -> String;
}
