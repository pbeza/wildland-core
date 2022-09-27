use std::sync::Arc;

use tokio::runtime::Runtime;
use wildland_corex::EncryptingKeypair;
use wildland_http_client::{
    error::WildlandHttpClientError,
    evs::{ConfirmTokenReq, EvsClient, GetStorageReq, GetStorageRes},
};

pub trait GetStorageResHandler: Sync {
    fn callback(&self, handler: FreeTierResp);
}

#[derive(Clone)]
pub struct FreeTierResp {
    encrypting_keypair: Arc<EncryptingKeypair>,
    storage_res: Result<GetStorageRes, WildlandHttpClientError>,
    evs_client: EvsClient,
    rt: Arc<Runtime>,
    email: String,
}

pub type WildlandHttpResult<T> = Result<T, WildlandHttpClientError>;

impl FreeTierResp {
    pub fn verification_handle(&self) -> WildlandHttpResult<FreeTierVerification> {
        self.storage_res.clone().map(|_resp| {
            // TODO
            FreeTierVerification {
                evs_client: self.evs_client.clone(),
                rt: self.rt.clone(),
                email: self.email.clone(),
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct FreeTierVerification {
    evs_client: EvsClient,
    rt: Arc<Runtime>,
    email: String,
}

impl FreeTierVerification {
    pub fn verify_email(
        &self,
        verification_token: String,
        resp_handler: &'static dyn ConfirmTokenResHandler,
    ) {
        self.rt.spawn({
            let evs_client = self.evs_client.clone();
            let email = self.email.clone();
            async move {
                let resp = evs_client
                    .confirm_token(ConfirmTokenReq {
                        email,
                        verification_token,
                    })
                    .await;
                resp_handler.callback(ConfirmTokenResp { confirm_res: resp })
            }
        });
    }
}

#[derive(Clone)]
pub struct ConfirmTokenResp {
    confirm_res: Result<(), WildlandHttpClientError>,
}

impl ConfirmTokenResp {
    pub fn check(&self) -> Result<(), WildlandHttpClientError> {
        self.confirm_res.clone()
    }
}

pub trait ConfirmTokenResHandler: Sync {
    fn callback(&self, handler: ConfirmTokenResp);
}

#[derive(Clone)]
pub struct FoundationStorageApi {
    rt: Arc<Runtime>,
    evs_client: EvsClient,
}

pub struct FoundationStorageApiConfiguration {
    pub evs_url: String,
}

impl FoundationStorageApi {
    pub fn new(config: FoundationStorageApiConfiguration) -> Self {
        Self {
            rt: Arc::new(Runtime::new().unwrap()), // TODO unwrap
            evs_client: EvsClient::new(&config.evs_url),
        }
    }

    pub fn request_free_tier_storage(
        &self,
        email: String,
        resp_handler: &'static dyn GetStorageResHandler,
    ) {
        let encrypting_keypair = EncryptingKeypair::new();
        let encoded_pub_key = encrypting_keypair.encode_pub();
        self.rt.spawn({
            let evs_client = self.evs_client.clone();
            let rt = self.rt.clone();
            async move {
                let resp = evs_client
                    .get_storage(GetStorageReq {
                        email: email.clone(),
                        pubkey: encoded_pub_key,
                    })
                    .await;
                resp_handler.callback(FreeTierResp {
                    encrypting_keypair: Arc::new(encrypting_keypair),
                    storage_res: resp,
                    evs_client,
                    rt,
                    email,
                });
            }
        });
    }
}
