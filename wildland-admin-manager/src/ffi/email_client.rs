use crate::api::{EmailClient, MockEmailClient};
use std::sync::Arc;

pub type BoxedDynEmailClient = Box<DynEmailClient>;

pub struct DynEmailClient(pub Arc<dyn EmailClient>);

pub fn create_boxed_mock_email_client() -> BoxedDynEmailClient {
    Box::new(DynEmailClient(Arc::new(MockEmailClient::new())))
}

pub fn create_mock_email_client() -> DynEmailClient {
    DynEmailClient(Arc::new(MockEmailClient::new()))
}
