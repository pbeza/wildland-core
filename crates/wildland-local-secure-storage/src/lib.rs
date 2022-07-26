mod api;
mod error;
mod file;

pub use api::LocalSecureStorage;
pub use error::LSSError;
pub use file::FileLSS;

pub type LSSResult<T> = Result<T, LSSError>;
