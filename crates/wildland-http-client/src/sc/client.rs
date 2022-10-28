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

use serde::Serialize;

use wildland_crypto::identity::signing_keypair::SigningKeypair;

use crate::{
    error::WildlandHttpClientError,
    response_handler::check_status_code,
    sc::credentials::{CreateCredentialsReq, CreateCredentialsRes, SCCredentialsClient},
    sc::metrics::{RequestMetricsReq, RequestMetricsRes, SCMetricsClient},
    sc::signature::{SCSignatureClient, SignatureRequestReq, SignatureRequestRes},
    sc::storage::{CreateStorageRes, SCStorageClient},
};

#[derive(Debug)]
pub struct Credentials {
    pub id: String,
    pub secret: String,
}

#[derive(Clone, Default, Debug)]
pub struct StorageControllerClient {
    // TODO:WILX-210 credentials are provided here only for test purposes. Remove it and get real id and secret assigned to a lease
    pub credential_id: String,
    pub credential_secret: String,
    sc_storage_client: SCStorageClient,
    sc_signature_client: SCSignatureClient,
    sc_credentials_client: SCCredentialsClient,
    sc_metrics_client: SCMetricsClient,
}

impl StorageControllerClient {
    #[tracing::instrument(level = "debug", ret)]
    pub fn new(base_url: &str) -> Self {
        Self {
            sc_storage_client: SCStorageClient {
                base_url: base_url.into(),
            },
            sc_signature_client: SCSignatureClient {
                base_url: base_url.into(),
            },
            sc_credentials_client: SCCredentialsClient {
                base_url: base_url.into(),
            },
            sc_metrics_client: SCMetricsClient {
                base_url: base_url.into(),
            },
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn create_storage(&self) -> Result<CreateStorageRes, WildlandHttpClientError> {
        let response = self.sc_storage_client.create_storage().map_err(Rc::new)?;
        let response_json = check_status_code(response)?.json().map_err(Rc::new)?;
        Ok(response_json)
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn create_credentials(
        &self,
        request: CreateCredentialsReq,
    ) -> Result<CreateCredentialsRes, WildlandHttpClientError> {
        let signature = self.sign_request(&request)?;
        let response = self
            .sc_credentials_client
            .create_credentials(request, &signature)
            .map_err(Rc::new)?;
        let response_json = check_status_code(response)?.json().map_err(Rc::new)?;
        Ok(response_json)
    }

    #[tracing::instrument(level = "debug", ret, skip(self, request))]
    pub async fn request_signature(
        &self,
        request: SignatureRequestReq,
    ) -> Result<SignatureRequestRes, WildlandHttpClientError> {
        let signature = self.sign_request(&request)?;
        let response = self
            .sc_signature_client
            .signature_request(request, &signature)
            .map_err(Rc::new)?;
        let response_json = check_status_code(response)?.json().map_err(Rc::new)?;
        Ok(response_json)
    }

    #[tracing::instrument(level = "debug", ret, skip(self, request))]
    pub async fn request_metrics(
        &self,
        request: RequestMetricsReq,
    ) -> Result<RequestMetricsRes, WildlandHttpClientError> {
        let signature = self.sign_request(&request)?;
        let response = self
            .sc_metrics_client
            .request_metrics(request, &signature)
            .map_err(Rc::new)?;
        let response_json = check_status_code(response)?.json().map_err(Rc::new)?;
        Ok(response_json)
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_credential_id(&self) -> &str {
        &self.credential_id
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_credential_secret(&self) -> &str {
        &self.credential_secret
    }

    #[tracing::instrument(level = "debug", ret, skip(self, request))]
    fn sign_request<T>(&self, request: &T) -> Result<String, WildlandHttpClientError>
    where
        T: Serialize,
    {
        let message = serde_json::to_vec(request).map_err(|source| {
            WildlandHttpClientError::CannotSerializeRequestError {
                source: Rc::new(source),
            }
        })?;
        let keypair =
            SigningKeypair::try_from_str(self.get_credential_id(), self.get_credential_secret())?;
        let signature = keypair.sign(&message);
        Ok(signature.encode_signature())
    }
}
