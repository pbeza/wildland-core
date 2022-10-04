use std::rc::Rc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::runtime::Runtime;
use uuid::Uuid;
use wildland_corex::{CryptoError, EncryptingKeypair};
use wildland_http_client::{
    error::WildlandHttpClientError,
    evs::{ConfirmTokenReq, DebugGetTokenReq, DebugProvisionReq, EvsClient, GetStorageReq},
};

use crate::FoundationStorageApiConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageCredentials {
    pub id: Uuid,
    #[serde(rename = "credentialID")]
    credential_id: String,
    #[serde(rename = "credentialSecret")]
    credential_secret: String,
}

#[derive(Clone)]
pub struct FoundationStorageApi {
    api_impl: Rc<dyn FoundationStorageApiImpl>,
}

trait FoundationStorageApiImpl {
    fn request_free_tier_storage(&self, email: String) -> Result<FreeTierProcessHandle, FsaError>;
    fn verify_email(
        &self,
        process_handle: &FreeTierProcessHandle,
        verification_token: String,
    ) -> Result<(), FsaError>;
}

#[repr(C)]
#[derive(Error, Debug, Clone)]
pub enum FsaError {
    #[error("Storage already exists for given email address")]
    StorageAlreadyExists,
    #[error("Evs Error: {0}")]
    EvsError(WildlandHttpClientError),
    #[error("Crypto error: {0}")]
    CryptoError(CryptoError),
    #[error("Credentials are expected to be JSON with fields: id, credentialID, credentialSecret")]
    InvalidCredentialsFormat(String),
}

#[derive(Clone)]
pub struct FreeTierProcessHandle {
    email: String,
    encrypting_keypair: Rc<EncryptingKeypair>,
}

impl FoundationStorageApi {
    pub fn new(config: FoundationStorageApiConfig) -> Self {
        let rt = Rc::new(Runtime::new().expect("Could not initialize tokio multithreaded runtime"));
        match config {
            FoundationStorageApiConfig::Prod { evs_url } => Self {
                api_impl: Rc::new(FoundationStorageApiProdImpl {
                    rt,
                    evs_client: EvsClient::new(&evs_url),
                }),
            },
            FoundationStorageApiConfig::Debug {
                evs_url,
                evs_credentials_payload,
            } => Self {
                api_impl: Rc::new(FoundationStorageApiDebugImpl {
                    rt,
                    evs_client: EvsClient::new(&evs_url),
                    payload: evs_credentials_payload,
                }),
            },
        }
    }

    pub fn request_free_tier_storage(
        &self,
        email: String,
    ) -> Result<FreeTierProcessHandle, FsaError> {
        self.api_impl.request_free_tier_storage(email)
    }

    pub fn verify_email(
        &self,
        process_handle: &FreeTierProcessHandle,
        verification_token: String,
    ) -> Result<(), FsaError> {
        self.api_impl
            .verify_email(process_handle, verification_token)
    }
}

struct FoundationStorageApiProdImpl {
    rt: Rc<Runtime>,
    evs_client: EvsClient,
}

impl FoundationStorageApiImpl for FoundationStorageApiProdImpl {
    fn request_free_tier_storage(&self, email: String) -> Result<FreeTierProcessHandle, FsaError> {
        self.rt
            .block_on(request_free_tier_storage(email, &self.evs_client))
    }

    fn verify_email(
        &self,
        process_handle: &FreeTierProcessHandle,
        verification_token: String,
    ) -> Result<(), FsaError> {
        self.rt.block_on(confirm_token_and_get_storage(
            &self.evs_client,
            &process_handle,
            verification_token,
        ))
    }
}

struct FoundationStorageApiDebugImpl {
    rt: Rc<Runtime>,
    evs_client: EvsClient,
    payload: String,
}

impl FoundationStorageApiImpl for FoundationStorageApiDebugImpl {
    fn request_free_tier_storage(&self, email: String) -> Result<FreeTierProcessHandle, FsaError> {
        self.rt
            .block_on(request_free_tier_storage(email, &self.evs_client))
    }

    fn verify_email(
        &self,
        process_handle: &FreeTierProcessHandle,
        verification_token: String,
    ) -> Result<(), FsaError> {
        let _ = verification_token;
        self.rt.block_on(async {
            let verification_token = self
                .evs_client
                .debug_get_token(DebugGetTokenReq {
                    pubkey: process_handle.encrypting_keypair.encode_pub(),
                    email: process_handle.email.clone(),
                })
                .await
                .map_err(FsaError::EvsError)?;

            self.evs_client
                .debug_provision(DebugProvisionReq {
                    email: process_handle.email.clone(),
                    payload: self.payload.clone(),
                })
                .await
                .map_err(FsaError::EvsError)?;

            confirm_token_and_get_storage(&self.evs_client, &process_handle, verification_token)
                .await
        })
    }
}

async fn request_free_tier_storage(
    email: String,
    evs_client: &EvsClient,
) -> Result<FreeTierProcessHandle, FsaError> {
    let encrypting_keypair = EncryptingKeypair::new();
    evs_client
        .get_storage(GetStorageReq {
            email: email.clone(),
            pubkey: encrypting_keypair.encode_pub(),
        })
        .await
        .map_err(FsaError::EvsError)
        .and_then(|resp| match resp.encrypted_credentials {
            Some(_) => Err(FsaError::StorageAlreadyExists),
            None => Ok(FreeTierProcessHandle {
                email,
                encrypting_keypair: Rc::new(encrypting_keypair),
            }),
        })
}

async fn confirm_token_and_get_storage(
    evs_client: &EvsClient,
    process_handle: &FreeTierProcessHandle,
    verification_token: String,
) -> Result<(), FsaError> {
    evs_client
        .confirm_token(ConfirmTokenReq {
            email: process_handle.email.clone(),
            verification_token,
        })
        .await
        .map_err(FsaError::EvsError)?;

    evs_client
        .get_storage(GetStorageReq {
            email: process_handle.email.clone(),
            pubkey: process_handle.encrypting_keypair.encode_pub(),
        })
        .await
        .map_err(FsaError::EvsError)
        .and_then(|resp| match resp.encrypted_credentials {
            Some(payload) => {
                let payload = payload.replace('\n', ""); // make sure that content is properly encoded
                let decoded = base64::decode(payload).unwrap();
                let decrypted = process_handle
                    .encrypting_keypair
                    .decrypt(decoded)
                    .map_err(|e| FsaError::CryptoError(e))?;
                let _storage_credentials: StorageCredentials =
                    serde_json::from_slice(&decrypted)
                        .map_err(|e| FsaError::InvalidCredentialsFormat(e.to_string()))?;
                // TODO do sth with credentials
                Ok(())
            }
            None => Err(FsaError::EvsError(WildlandHttpClientError::NoBody)),
        })
}
