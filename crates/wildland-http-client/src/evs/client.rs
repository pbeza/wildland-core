use std::sync::Arc;

use crate::{error::WildlandHttpClientError, response_handler::handle};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmTokenReq {
    pub email: String,
    pub verification_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStorageReq {
    pub email: String,
    pub pubkey: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetStorageRes {
    pub encrypted_credentials: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugGetTokenReq {
    pub email: String,
    pub pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugProvisionReq {
    pub email: String,
    pub payload: String,
}

#[derive(Clone, Default, Debug)]
pub struct EvsClient {
    base_url: String,
    client: Client,
}

impl EvsClient {
    #[tracing::instrument(level = "debug", ret)]
    pub fn new(base_url: &str) -> Self {
        let client = Client::new();
        Self {
            base_url: base_url.to_string(),
            client: client,
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn confirm_token(
        &self,
        request: ConfirmTokenReq,
    ) -> Result<(), WildlandHttpClientError> {
        let url = format!("{}/confirm_token", self.base_url);
        let response = self
            .client
            .put(url)
            .json(&request)
            .send()
            .await
            .map_err(Arc::new)?;
        handle(response).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn get_storage(
        &self,
        request: GetStorageReq,
    ) -> Result<GetStorageRes, WildlandHttpClientError> {
        let url = format!("{}/get_storage", self.base_url);
        let response = self
            .client
            .put(url)
            .json(&request)
            .send()
            .await
            .map_err(Arc::new)?;
        let response = handle(response).await?;
        match response {
            Some(response) => Ok(response.json().await.map_err(Arc::new)?),
            None => Ok(GetStorageRes {
                encrypted_credentials: None,
            }),
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn debug_get_token(
        &self,
        request: DebugGetTokenReq,
    ) -> Result<String, WildlandHttpClientError> {
        let url = format!("{}/debug_get_token", self.base_url);
        let response = self
            .client
            .get(url)
            .query(&[("email", request.email), ("pubkey", request.pubkey)])
            .send()
            .await
            .map_err(Arc::new)?;
        let response = handle(response).await?;
        match response {
            Some(response) => Ok(String::from_utf8(
                response
                    .bytes()
                    .await
                    .map_err(|_| {
                        WildlandHttpClientError::HttpError(
                            "Could not get response bytes".to_owned(),
                        )
                    })?
                    .to_vec(),
            )
            .map_err(|_| {
                WildlandHttpClientError::HttpError(
                    "Could not parse response body as utf-8 string".to_owned(),
                )
            })?),
            None => Err(WildlandHttpClientError::NoBody),
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn debug_provision(
        &self,
        request: DebugProvisionReq,
    ) -> Result<(), WildlandHttpClientError> {
        let url = format!("{}/debug_provision", self.base_url);
        let response = self
            .client
            .put(url)
            .query(&[("email", request.email), ("credentials", request.payload)])
            .send()
            .await
            .map_err(Arc::new)?;
        handle(response).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::evs::constants::test_utilities::{
        EMAIL, ENCRYPTED_CREDENTIALS, PUBKEY, VERIFICATION_TOKEN,
    };
    use mockito::{mock, server_url};
    use serde_json::json;

    use super::*;

    fn client() -> EvsClient {
        EvsClient {
            base_url: server_url(),
            client: Client::new(),
        }
    }

    #[tokio::test]
    async fn should_confirm_token() {
        // given
        let request = ConfirmTokenReq {
            email: EMAIL.into(),
            verification_token: VERIFICATION_TOKEN.into(),
        };

        let m = mock("PUT", "/confirm_token").create();

        // when
        let response = client()._confirm_token(request).await;

        // then
        m.assert();
        response.unwrap();
    }

    #[tokio::test]
    async fn should_get_storage() {
        // given
        let request = GetStorageReq {
            email: EMAIL.into(),
            pubkey: PUBKEY.into(),
        };

        let m = mock("PUT", "/get_storage")
            .with_body(json!({ "encrypted_credentials": ENCRYPTED_CREDENTIALS }).to_string())
            .create();

        // when
        let response = client().get_storage(request).await.unwrap();

        // then
        m.assert();
        assert_eq!(response.encrypted_credentials, ENCRYPTED_CREDENTIALS);
    }
}
