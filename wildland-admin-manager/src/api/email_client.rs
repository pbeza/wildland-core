use super::AdminManagerResult;
#[cfg(any(feature = "mocks", test))]
use mockall::automock;

#[cfg_attr(any(feature = "mocks", test), automock)]
pub trait EmailClient {
    fn send(&self, address: &str, message: &str) -> AdminManagerResult<()>;
}
