//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::rc::Rc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use wildland_corex::{
    storage::StorageTemplateTrait, CryptoError, EncryptingKeypair, LssError, LssService,
};
use wildland_http_client::{
    error::WildlandHttpClientError,
    evs::{ConfirmTokenReq, EvsClient, GetStorageReq},
};

use super::{config::FoundationStorageApiConfig, storage_template::StorageTemplate};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageCredentials {
    pub id: Uuid,
    #[serde(rename = "credentialID")]
    credential_id: String,
    #[serde(rename = "credentialSecret")]
    credential_secret: String,
}

impl StorageCredentials {
    fn into_storage_template(self, sc_url: String) -> FoundationStorageTemplate {
        FoundationStorageTemplate {
            id: self.id,
            credential_id: self.credential_id,
            credential_secret: self.credential_secret,
            sc_url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundationStorageTemplate {
    pub id: Uuid,
    pub credential_id: String,
    pub credential_secret: String,
    pub sc_url: String,
}

impl StorageTemplateTrait for FoundationStorageTemplate {
    fn uuid(&self) -> Uuid {
        self.id
    }
}

impl From<FoundationStorageTemplate> for StorageTemplate {
    fn from(fst: FoundationStorageTemplate) -> Self {
        Self::FoundationStorageTemplate(fst)
    }
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
    #[error(
        "Credentials are expected to be JSON with fields: id, credentialID, credentialSecret: {0}"
    )]
    InvalidCredentialsFormat(String),
    #[error(transparent)]
    LssError(#[from] LssError),
}

#[derive(Clone)]
pub struct FreeTierProcessHandle {
    email: String,
    encrypting_keypair: Rc<EncryptingKeypair>,
}

#[derive(Clone)]
pub struct FoundationStorageApi {
    evs_client: EvsClient,
    sc_url: String,
    lss_service: LssService,
}

impl FoundationStorageApi {
    pub fn new(config: FoundationStorageApiConfig, lss_service: LssService) -> Self {
        Self {
            evs_client: EvsClient::new(&config.evs_url),
            sc_url: config.sc_url,
            lss_service,
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
    ) -> Result<StorageTemplate, FsaError> {
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
                    // TODO WILX-269 EVS communication encryption
                    // let decrypted_hex = process_handle
                    //     .encrypting_keypair
                    //     .decrypt(decoded)
                    //     .map_err(FsaError::CryptoError)?;
                    // let decrypted = hex::decode(decrypted_hex).unwrap();
                    let decrypted = decoded;
                    let storage_credentials: StorageCredentials =
                        serde_json::from_slice(&decrypted)
                            .map_err(|e| FsaError::InvalidCredentialsFormat(e.to_string()))?;

                    let storage_template = StorageTemplate::FoundationStorageTemplate(
                        storage_credentials.into_storage_template(self.sc_url.clone()),
                    );
                    self.lss_service.save_storage_template(&storage_template)?;

                    Ok(storage_template)
                }
                None => Err(FsaError::EvsError(WildlandHttpClientError::HttpError(
                    "No body with credentials".into(),
                ))),
            })
    }
}
