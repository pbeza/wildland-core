use std::rc::Rc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use wildland_corex::{CryptoError, EncryptingKeypair};
use wildland_http_client::{
    error::WildlandHttpClientError,
    evs::{ConfirmTokenReq, EvsClient, GetStorageReq},
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

#[derive(Clone)]
pub struct FoundationStorageApi {
    evs_client: EvsClient,
}

impl FoundationStorageApi {
    pub fn new(config: FoundationStorageApiConfig) -> Self {
        Self {
            evs_client: EvsClient::new(&config.evs_url),
        }
    }

    pub fn request_free_tier_storage(
        &self,
        email: String,
    ) -> Result<FreeTierProcessHandle, FsaError> {
        let encrypting_keypair = EncryptingKeypair::new();
        self.evs_client
            .get_storage(GetStorageReq {
                email: email.clone(),
                pubkey: encrypting_keypair.encode_pub(),
            })
            .map_err(FsaError::EvsError)
            .and_then(|resp| match resp.encrypted_credentials {
                Some(_) => Err(FsaError::StorageAlreadyExists),
                None => Ok(FreeTierProcessHandle {
                    email,
                    encrypting_keypair: Rc::new(encrypting_keypair),
                }),
            })
    }

    pub fn verify_email(
        &self,
        process_handle: &FreeTierProcessHandle,
        verification_token: String,
    ) -> Result<(), FsaError> {
        self.evs_client
            .confirm_token(ConfirmTokenReq {
                email: process_handle.email.clone(),
                verification_token,
            })
            .map_err(FsaError::EvsError)?;

        self.evs_client
            .get_storage(GetStorageReq {
                email: process_handle.email.clone(),
                pubkey: process_handle.encrypting_keypair.encode_pub(),
            })
            .map_err(FsaError::EvsError)
            .and_then(|resp| match resp.encrypted_credentials {
                Some(payload) => {
                    let payload = payload.replace('\n', ""); // make sure that content is properly encoded
                    let decoded = base64::decode(payload).unwrap();
                    let decrypted_hex = process_handle
                        .encrypting_keypair
                        .decrypt(decoded)
                        .map_err(FsaError::CryptoError)?;
                    let decrypted = hex::decode(decrypted_hex).unwrap();
                    let _storage_credentials: StorageCredentials =
                        serde_json::from_slice(&decrypted)
                            .map_err(|e| FsaError::InvalidCredentialsFormat(e.to_string()))?;
                    // TODO do sth with credentials
                    Ok(())
                }
                None => Err(FsaError::EvsError(WildlandHttpClientError::NoBody)),
            })
    }
}
