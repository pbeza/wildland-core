use reqwest::Client;
use serde::Serialize;

use wildland_crypto::{
    identity::keys::{SigningKeypair, Keypair},
    signature::encode_signature,
};

use crate::{
    credentials::{CreateCredentialsReq, CreateCredentialsRes, SCCredentialsClient},
    error::StorageControllerClientError,
    metrics::{RequestMetricsReq, RequestMetricsRes, SCMetricsClient},
    response_handler::handle,
    signature::{SCSignatureClient, SignatureRequestReq, SignatureRequestRes},
    storage::{CreateStorageRes, SCStorageClient},
};

pub struct Credentials {
    pub id: String,
    pub secret: String,
}

#[derive(Clone, Default)]
pub struct StorageControllerClient {
    // TODO credentials are provided here only for test purposes. Remove it and get real id and secret assigned to a lease
    pub credential_id: String,
    pub credential_secret: String,
    sc_storage_client: SCStorageClient,
    sc_signature_client: SCSignatureClient,
    sc_credentials_client: SCCredentialsClient,
    sc_metrics_client: SCMetricsClient,
}

impl StorageControllerClient {
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

    pub async fn create_storage(&self) -> Result<CreateStorageRes, StorageControllerClientError> {
        let response = self.sc_storage_client.create_storage().await?;
        let response_json = handle(response).await?.json().await?;
        Ok(response_json)
    }

    pub async fn create_credentials(
        &self,
        request: CreateCredentialsReq,
    ) -> Result<CreateCredentialsRes, StorageControllerClientError> {
        let signature = self.sign_request(&request)?;
        let response = self
            .sc_credentials_client
            .create_credentials(request, &signature)
            .await?;
        let response_json = handle(response).await?.json().await?;
        Ok(response_json)
    }

    pub async fn request_signature(
        &self,
        request: SignatureRequestReq,
    ) -> Result<SignatureRequestRes, StorageControllerClientError> {
        let signature = self.sign_request(&request)?;
        let response = self
            .sc_signature_client
            .signature_request(request, &signature)
            .await?;
        let response_json = handle(response).await?.json().await?;
        Ok(response_json)
    }

    pub async fn request_metrics(
        &self,
        request: RequestMetricsReq,
    ) -> Result<RequestMetricsRes, StorageControllerClientError> {
        let signature = self.sign_request(&request)?;
        let response = self
            .sc_metrics_client
            .request_metrics(request, &signature)
            .await?;
        let response_json = handle(response).await?.json().await?;
        Ok(response_json)
    }

    pub fn set_credentials(&mut self, credentials: Credentials) {
        self.credential_id = credentials.id;
        self.credential_secret = credentials.secret;
    }

    pub fn get_credential_id(&self) -> &str {
        &self.credential_id
    }

    pub fn get_credential_secret(&self) -> &str {
        &self.credential_secret
    }

    fn sign_request<T>(&self, request: &T) -> Result<String, StorageControllerClientError>
    where
        T: Serialize,
    {
        let message = serde_json::to_vec(request).map_err(|source| {
            StorageControllerClientError::CannotSerializeRequestError { source }
        })?;
        let keypair = SigningKeypair::from_str(
            self.get_credential_id(),
            self.get_credential_secret(),
        )?;
        let signature = keypair.sign(&message);
        Ok(encode_signature(signature))
    }
}
