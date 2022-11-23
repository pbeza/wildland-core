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
    catlib_service::error::CatlibError, storage::StorageTemplateTrait, CryptoError,
    EncryptingKeypair, LssError,
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
            uuid: self.id,
            credential_id: self.credential_id,
            credential_secret: self.credential_secret,
            sc_url,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundationStorageTemplate {
    pub uuid: Uuid,
    pub credential_id: String,
    pub credential_secret: String,
    pub sc_url: String,
}

impl StorageTemplateTrait for FoundationStorageTemplate {
    fn uuid(&self) -> Uuid {
        self.uuid
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
    #[error(transparent)]
    CatlibError(#[from] CatlibError),
    #[error("{0}")]
    Generic(String),
}

#[derive(Clone)]
pub struct FoundationStorageApi {
    evs_client: EvsClient,
    sc_url: String,
}

impl FoundationStorageApi {
    pub fn new(config: &FoundationStorageApiConfig) -> Self {
        Self {
            evs_client: EvsClient::new(&config.evs_url),
            sc_url: config.sc_url.clone(),
        }
    }

    #[tracing::instrument(level="debug", skip_all)]
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
                    evs_client: self.evs_client.clone(),
                    sc_url: self.sc_url.clone(),
                }),
            })
    }
}

/// Represents ongoing process of granting Free Foundation Storage and allows to run email verifications
/// via `verify_email` method.
#[derive(Clone)]
pub struct FreeTierProcessHandle {
    email: String,
    encrypting_keypair: Rc<EncryptingKeypair>, // TODO WILX-269 EVS communication encryption (remove it if choose to relay on SSL only)
    evs_client: EvsClient,
    sc_url: String,
}

impl FreeTierProcessHandle {
    /// Verifies user's email.
    /// After successful verification it returns Foundation Storage Template (which is also saved in LSS)
    /// and saves information in CatLib that Foundation storage has been granted.
    #[tracing::instrument(level="debug", skip_all)]
    pub fn verify_email(&self, verification_token: String) -> Result<StorageTemplate, FsaError> {
        self.evs_client
            .confirm_token(ConfirmTokenReq {
                email: self.email.clone(),
                verification_token,
            })
            .map_err(FsaError::EvsError)?;

        self.evs_client
            .get_storage(GetStorageReq {
                email: self.email.clone(),
                pubkey: self.encrypting_keypair.encode_pub(),
            })
            .map_err(FsaError::EvsError)
            .and_then(|resp| match resp.encrypted_credentials {
                Some(payload) => {
                    let payload = payload.replace('\n', ""); // make sure that content is properly encoded
                    let decoded = base64::decode(payload).unwrap();
                    let storage_credentials: StorageCredentials = serde_json::from_slice(&decoded)
                        .map_err(|e| FsaError::InvalidCredentialsFormat(e.to_string()))?;

                    let storage_template = StorageTemplate::FoundationStorageTemplate(
                        storage_credentials.into_storage_template(self.sc_url.clone()),
                    );

                    Ok(storage_template)
                }
                None => Err(FsaError::EvsError(WildlandHttpClientError::HttpError(
                    "No body with credentials".into(),
                ))),
            })
    }
}
