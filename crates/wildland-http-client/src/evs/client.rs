use std::rc::Rc;

use crate::{error::WildlandHttpClientError, response_handler::handle};
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

#[derive(Clone, Default, Debug)]
pub struct EvsClient {
    base_url: String,
}

impl EvsClient {
    #[tracing::instrument(level = "debug", ret)]
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn confirm_token(&self, request: ConfirmTokenReq) -> Result<(), WildlandHttpClientError> {
        let url = format!("{}/confirm_token", self.base_url);
        let response = minreq::put(url)
            .with_json(&request)
            .map_err(|e| WildlandHttpClientError::HttpLibError(Rc::new(e)))?
            .send()
            .map_err(Rc::new)?;
        handle(response)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_storage(
        &self,
        request: GetStorageReq,
    ) -> Result<GetStorageRes, WildlandHttpClientError> {
        let url = format!("{}/get_storage", self.base_url);
        let response = minreq::put(url)
            .with_json(&request)
            .map_err(|e| WildlandHttpClientError::HttpLibError(Rc::new(e)))?
            .send()
            .map_err(Rc::new)?;
        let response = handle(response)?;
        match response {
            Some(response) => Ok(response.json().map_err(Rc::new)?),
            None => Ok(GetStorageRes {
                encrypted_credentials: None,
            }),
        }
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
        }
    }

    #[test]
    fn should_confirm_token() {
        // given
        let request = ConfirmTokenReq {
            email: EMAIL.into(),
            verification_token: VERIFICATION_TOKEN.into(),
        };

        let m = mock("PUT", "/confirm_token").create();

        // when
        let response = client().confirm_token(request);

        // then
        m.assert();
        response.unwrap();
    }

    #[test]
    fn should_get_storage() {
        // given
        let request = GetStorageReq {
            email: EMAIL.into(),
            pubkey: PUBKEY.into(),
        };

        let m = mock("PUT", "/get_storage")
            .with_body(json!({ "encrypted_credentials": ENCRYPTED_CREDENTIALS }).to_string())
            .create();

        // when
        let response = client().get_storage(request).unwrap();

        // then
        m.assert();
        assert_eq!(
            response.encrypted_credentials.unwrap(),
            ENCRYPTED_CREDENTIALS
        );
    }
}
