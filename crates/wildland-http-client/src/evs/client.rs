use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::error::WildlandHttpClientError;
use crate::response_handler::handle;


#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmTokenReq {
    pub email: String,
    pub verification_token: String,
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
            client
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub async fn confirm_token(&self, request: ConfirmTokenReq) -> Result<(), WildlandHttpClientError> {
        let url = format!("{}/confirm_token", self.base_url);
        let response = self.client.put(url).json(&request).send().await?;
        handle(response).await?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use mockito::{mock, server_url};
    use serde_json::json;
    use crate::evs::constants::test_utilities::{EMAIL, VERIFICATION_TOKEN};

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
            .with_body(json!({ "email": EMAIL, "verification_token": VERIFICATION_TOKEN }).to_string())
            .create();

        // when
        let response = client()
            .confirm_token(request)
            .await;

        // then
        m.assert();
        response.unwrap();
    }
}
