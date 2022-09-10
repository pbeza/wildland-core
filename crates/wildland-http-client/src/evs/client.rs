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
    _base_url: String,
    _client: Client,
}

impl EVSClient {
    #[tracing::instrument(level = "debug", ret)]
    pub fn _new(base_url: &str) -> Self {
        let client = Client::new();
        Self {
            _base_url: base_url.to_string(),
            _client: client,
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn _confirm_token(
        &self,
        request: ConfirmTokenReq,
    ) -> Result<(), WildlandHttpClientError> {
        let url = format!("{}/confirm_token", self._base_url);
        let response = self._client.put(url).json(&request).send().await?;
        handle(response).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn _get_storage(
        &self,
        request: GetStorageReq,
    ) -> Result<GetStorageRes, WildlandHttpClientError> {
        let url = format!("{}/get_storage", self._base_url);
        let response = self._client.put(url).json(&request).send().await?;
        let response_json = handle(response).await?.json().await?;
        Ok(response_json)
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

    fn client() -> EVSClient {
        EVSClient {
            _base_url: server_url(),
            _client: Client::new(),
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
        let response = client()._get_storage(request).await.unwrap();

        // then
        m.assert();
        assert_eq!(response.encrypted_credentials, ENCRYPTED_CREDENTIALS);
    }
}
