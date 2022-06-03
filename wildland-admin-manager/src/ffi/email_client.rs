use crate::api::{EmailClient, MockEmailClient};
use std::sync::Arc;

pub type BoxedDynEmailClient = Box<DynEmailClient>;

pub struct DynEmailClient(pub Arc<dyn EmailClient>);

// TODO feature flag

pub fn create_boxed_email_client_mock_builder() -> Box<EmailClientMockBuilder> {
    Box::new(create_email_client_mock_builder())
}

pub fn create_email_client_mock_builder() -> EmailClientMockBuilder {
    EmailClientMockBuilder(Arc::new(MockEmailClient::new()))
}

pub struct EmailClientMockBuilder(pub Arc<MockEmailClient>);
impl EmailClientMockBuilder {
    pub fn expect_send(&mut self, address: String, message: String, times: usize) {
        Arc::get_mut(&mut self.0)
            .unwrap()
            .expect_send()
            .times(times)
            .withf(move |addr, code| addr == address && code == message)
            .returning(|_, _| Ok(()));
    }

    pub fn build_boxed(&self) -> BoxedDynEmailClient {
        Box::new(self.build())
    }

    pub fn build(&self) -> DynEmailClient {
        DynEmailClient(self.0.clone())
    }
}
