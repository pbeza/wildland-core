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

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use wildland_corex::catlib_service::error::CatlibError;
use wildland_corex::{CryptoError, LssError, StorageTemplate, StorageTemplateError};
use wildland_http_client::error::WildlandHttpClientError;
use wildland_http_client::evs::{ConfirmTokenReq, EvsClient, GetStorageReq, GetStorageRes};

use super::config::FoundationStorageApiConfig;
use crate::templates::foundation_storage::FoundationStorageTemplate;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageCredentials {
    pub id: Uuid,
    #[serde(rename = "credentialID")]
    pub credential_id: String,
    #[serde(rename = "credentialSecret")]
    pub credential_secret: String,
}

/// Errors that may happen during using Foundation Storage API (communication with EVS server)
///
#[repr(C)]
#[derive(Error, Debug, Clone)]
pub enum FsaError {
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
    #[error("Error while creating Storage Template: {0}")]
    StorageTemplateError(StorageTemplateError),
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

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn request_free_tier_storage(
        &self,
        email: String,
    ) -> Result<FreeTierProcessHandle, FsaError> {
        self.evs_client
            .get_storage(GetStorageReq {
                email: email.clone(),
                session_id: None,
            })
            .map_err(FsaError::EvsError)
            .and_then(|resp| match resp {
                GetStorageRes {
                    session_id: None, ..
                } => Err(FsaError::Generic(
                    "EVS did not return expected session id".to_string(),
                )),
                GetStorageRes {
                    session_id: Some(session_id),
                    ..
                } => Ok(FreeTierProcessHandle {
                    email,
                    session_id,
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
    session_id: String,
    evs_client: EvsClient,
    sc_url: String,
}

impl FreeTierProcessHandle {
    /// Verifies user's email.
    /// After successful verification it returns Foundation Storage Template (which is also saved in LSS)
    /// and saves information in CatLib that Foundation storage has been granted.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn verify_email(&self, verification_token: String) -> Result<StorageTemplate, FsaError> {
        self.evs_client
            .confirm_token(ConfirmTokenReq {
                session_id: self.session_id.clone(),
                email: self.email.clone(),
                verification_token,
            })
            .map_err(FsaError::EvsError)?;

        self.evs_client
            .get_storage(GetStorageReq {
                email: self.email.clone(),
                session_id: Some(self.session_id.clone()),
            })
            .map_err(FsaError::EvsError)
            .and_then(|resp| match resp.credentials {
                Some(payload) => {
                    let decoded = base64::decode(payload).map_err(|e| {
                        FsaError::Generic(format!(
                            "EVS returned incorrectly formatted base64 credentials: {e}"
                        ))
                    })?;
                    let storage_credentials: StorageCredentials = serde_json::from_slice(&decoded)
                        .map_err(|e| FsaError::InvalidCredentialsFormat(e.to_string()))?;

                    let storage_template: StorageTemplate =
                        FoundationStorageTemplate::from_storage_credentials_and_sc_url(
                            storage_credentials,
                            self.sc_url.clone(),
                        )
                        .try_into()
                        .map_err(FsaError::StorageTemplateError)?;

                    Ok(storage_template)
                }
                None => Err(FsaError::EvsError(WildlandHttpClientError::HttpError(
                    "No body with credentials".into(),
                ))),
            })
    }
}
