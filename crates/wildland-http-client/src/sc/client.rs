use std::sync::Arc;

use reqwest::Client;
use serde::Serialize;

use wildland_crypto::identity::signing_keypair::SigningKeypair;

use crate::{
    error::WildlandHttpClientError,
    response_handler::handle,
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
        let client = Client::new();
        Self {
            sc_storage_client: SCStorageClient {
                base_url: base_url.into(),
                client: client.clone(),
            },
            sc_signature_client: SCSignatureClient {
                base_url: base_url.into(),
                client: client.clone(),
            },
            sc_credentials_client: SCCredentialsClient {
                base_url: base_url.into(),
                client: client.clone(),
            },
            sc_metrics_client: SCMetricsClient {
                base_url: base_url.into(),
                client,
            },
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn create_storage(&self) -> Result<CreateStorageRes, WildlandHttpClientError> {
        let response = self
            .sc_storage_client
            .create_storage()
            .await
            .map_err(Arc::new)?;
        let response_json = handle(response)
            .await?
            .ok_or(WildlandHttpClientError::NoBody)?
            .json()
            .await
            .map_err(Arc::new)?;
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
            .await
            .map_err(Arc::new)?;
        let response_json = handle(response)
            .await?
            .ok_or(WildlandHttpClientError::NoBody)?
            .json()
            .await
            .map_err(Arc::new)?;
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
            .await
            .map_err(Arc::new)?;
        let response_json = handle(response)
            .await?
            .ok_or(WildlandHttpClientError::NoBody)?
            .json()
            .await
            .map_err(Arc::new)?;
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
            .await
            .map_err(Arc::new)?;
        let response_json = handle(response)
            .await?
            .ok_or(WildlandHttpClientError::NoBody)?
            .json()
            .await
            .map_err(Arc::new)?;
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
                source: Arc::new(source),
            }
        })?;
        let keypair =
            SigningKeypair::try_from_str(self.get_credential_id(), self.get_credential_secret())?;
        let signature = keypair.sign(&message);
        Ok(signature.encode_signature())
    }
}
