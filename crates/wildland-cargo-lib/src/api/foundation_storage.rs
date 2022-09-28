use std::sync::Arc;

use tokio::runtime::Runtime;
use wildland_corex::EncryptingKeypair;
use wildland_http_client::{
    error::WildlandHttpClientError,
    evs::{ConfirmTokenReq, DebugGetTokenReq, EvsClient, GetStorageReq, GetStorageRes},
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
        log::info!("Checking response of method getting storage");
        self.storage_res.clone().map(|resp| {
            log::debug!("{resp:?}"); // TODO remove
            if let Some(encrypted_credentials) = resp.encrypted_credentials {
                let decrypted_credentials = self
                    .encrypting_keypair
                    .decrypt(encrypted_credentials.into());
                println!("DECRYPTED: {decrypted_credentials:?}"); // TODO remove
            }

            FreeTierVerification {
                encrypting_keypair: self.encrypting_keypair.clone(),
                evs_client: self.evs_client.clone(),
                rt: self.rt.clone(),
                email: self.email.clone(),
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct FreeTierVerification {
    encrypting_keypair: Arc<EncryptingKeypair>,
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
        log::info!("Verifying email");

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
                log::debug!("{resp:?}");
                resp_handler.callback(ConfirmTokenResp { confirm_res: resp });
            }
        });
    }

    // TODO hide behind feature flag like "debug_query"
    pub fn debug_get_token(&self, resp_handler: &'static dyn DebugGetTokenResHandler) {
        log::info!("Debug token retrieval");
        self.rt.spawn({
            let evs_client = self.evs_client.clone();
            let email = self.email.clone();
            let pubkey = self.encrypting_keypair.encode_pub();
            async move {
                let resp = evs_client
                    .debug_get_token(DebugGetTokenReq { email, pubkey })
                    .await;
                log::debug!("{resp:?}");
                resp_handler.callback(DebugGetTokenResp {
                    get_token_res: resp,
                })
            }
        });
    }
}

#[derive(Clone)]
pub struct DebugGetTokenResp {
    get_token_res: Result<String, WildlandHttpClientError>,
}

impl DebugGetTokenResp {
    pub fn get_token(&self) -> Result<String, WildlandHttpClientError> {
        self.get_token_res.clone()
    }
}

pub trait DebugGetTokenResHandler: Sync {
    fn callback(&self, handler: DebugGetTokenResp);
}

#[derive(Clone)]
pub struct ConfirmTokenResp {
    confirm_res: Result<(), WildlandHttpClientError>,
}

impl ConfirmTokenResp {
    pub fn check(&self) -> Result<(), WildlandHttpClientError> {
        log::info!("Checking response of token verification");
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
