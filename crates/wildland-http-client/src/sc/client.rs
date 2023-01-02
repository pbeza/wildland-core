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

use super::constants::WILDLAND_SIGNATURE_HEADER;
use super::models::{
    CreateCredentialsReq, CreateCredentialsRes, CreateStorageRes, RequestMetricsReq,
    RequestMetricsRes, SignatureRequestReq, SignatureRequestRes,
};
use crate::cross_platform_http_client::{CurrentPlatformClient, HttpClient, Request};
use crate::error::WildlandHttpClientError;
use crate::response_handler::check_status_code;

#[derive(Debug)]
pub struct Credentials {
    pub id: String,
    pub secret: String,
}

#[derive(Clone)]
pub struct StorageControllerClient {
    // TODO:WILX-210 credentials are provided here only for test purposes. Remove it and get real id and secret assigned to a lease
    pub credential_id: String,
    pub credential_secret: String,
    http_client: Rc<dyn HttpClient>,
}

impl StorageControllerClient {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(base_url: &str) -> Self {
        let http_client = Rc::new(CurrentPlatformClient {
            base_url: base_url.into(),
        });

        Self {
            credential_id: String::default(),
            credential_secret: String::default(),
            http_client,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_storage(&self) -> Result<CreateStorageRes, WildlandHttpClientError> {
        let request = Request::new("/storage/create");
        let response = self.http_client.post(request)?;
        let response = check_status_code(response)?;
        Ok(response.deserialize()?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_credentials(
        &self,
        request: CreateCredentialsReq,
    ) -> Result<CreateCredentialsRes, WildlandHttpClientError> {
        let signature = self.sign_request(&request)?;
        let http_request = Request::new("/credential/create")
            .with_json(&request)
            .with_header(WILDLAND_SIGNATURE_HEADER, signature);
        let response = self.http_client.post(http_request)?;
        let response = check_status_code(response)?;
        Ok(response.deserialize()?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn request_signature(
        &self,
        request: SignatureRequestReq,
    ) -> Result<SignatureRequestRes, WildlandHttpClientError> {
        let signature = self.sign_request(&request)?;
        let http_request = Request::new("/signature/request")
            .with_json(&request)
            .with_header(WILDLAND_SIGNATURE_HEADER, signature);
        let response = self.http_client.post(http_request)?;
        let response = check_status_code(response)?;
        Ok(response.deserialize()?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn request_metrics(
        &self,
        request: RequestMetricsReq,
    ) -> Result<RequestMetricsRes, WildlandHttpClientError> {
        let signature = self.sign_request(&request)?;
        let http_request = Request::new("/metrics")
            .with_json(&request)
            .with_header(WILDLAND_SIGNATURE_HEADER, signature);
        let response = self.http_client.post(http_request)?;
        let response = check_status_code(response)?;
        Ok(response.deserialize()?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_credential_id(&self) -> &str {
        &self.credential_id
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_credential_secret(&self) -> &str {
        &self.credential_secret
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn sign_request<T>(&self, request: &T) -> Result<String, WildlandHttpClientError>
    where
        T: Serialize,
    {
        let message = serde_json::to_vec(request).map_err(Rc::new)?;
        let keypair =
            SigningKeypair::try_from_str(self.get_credential_id(), self.get_credential_secret())?;
        let signature = keypair.sign(&message);
        Ok(signature.encode_signature())
    }
}
