use std::rc::Rc;

use tokio::runtime::Runtime;
use wildland_http_client::evs::{EvsClient, GetStorageReq};

#[derive(Clone)]
pub struct FoundationStorageApi {
    rt: Rc<Runtime>,
    evs_client: EvsClient,
}

pub struct FoundationStorageApiConfiguration {
    pub evs_url: String,
}

impl FoundationStorageApi {
    pub fn new(config: FoundationStorageApiConfiguration) -> Self {
        Self {
            rt: Rc::new(Runtime::new().unwrap()), // TODO unwrap
            evs_client: EvsClient::new(&config.evs_url),
        }
    }

    pub fn request_free_tier_storage(&self, email: String, pubkey: String) -> FreeTierProcess {
        let local_client = self.evs_client.clone();
        self.rt.spawn(async move {
            local_client
                .get_storage(GetStorageReq { email, pubkey })
                .await
        });

        FreeTierProcess
    }
}

pub struct FreeTierProcess;
