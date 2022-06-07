use super::AdminManagerResult;
use mockall::automock;

#[automock]
pub trait EmailClient {
    fn send(&self, address: &str, message: &str) -> AdminManagerResult<()>;
}
