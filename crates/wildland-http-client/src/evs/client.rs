use crate::error::WildlandHttpClientError;
use crate::response_handler::handle;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStorageRes {
    pub encrypted_credentials: String,
}

#[derive(Clone, Default, Debug)]
pub struct EVSClient {
    pub(crate) base_url: String,
    pub(crate) client: Client,
}

impl EVSClient {
    #[tracing::instrument(level = "debug", ret)]
    pub fn new(base_url: &str) -> Self {
        let client = Client::new();
        Self {
            base_url: base_url.to_string(),
            client,
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn confirm_token(
        &self,
        request: ConfirmTokenReq,
    ) -> Result<(), WildlandHttpClientError> {
        let url = format!("{}/confirm_token", self.base_url);
        let response = self.client.put(url).json(&request).send().await?;
        handle(response).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn get_storage(&self, request: GetStorageReq) -> Result<GetStorageRes, WildlandHttpClientError> {
        let url = format!("{}/get_storage", self.base_url);
        let response = self.client.put(url).json(&request).send().await?;
        let response_json = handle(response).await?.json().await?;
        Ok(response_json)
    }
}

#[cfg(test)]
mod tests {
    use crate::evs::constants::test_utilities::{EMAIL, ENCRYPTED_CREDENTIALS, PUBKEY, VERIFICATION_TOKEN};
    use mockito::{mock, server_url};
    use serde_json::json;

    use super::*;

    fn client() -> EVSClient {
        EVSClient {
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

        let m = mock("PUT", "/confirm_token")
            .create();

        // when
        let response = client().confirm_token(request).await;

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
            .with_body(
                json!({ "encrypted_credentials": ENCRYPTED_CREDENTIALS }).to_string(),
            )
            .create();

        // when
        let response = client()
            .get_storage(request)
            .await
            .unwrap();

        // then
        m.assert();
        assert_eq!(response.encrypted_credentials, ENCRYPTED_CREDENTIALS);
    }
}
