mod api;
mod file;

pub use api::LocalSecureStorage;
pub use file::FileLSS;

pub type LSSResult<T> = Result<T, String>; // TODO string?
