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

use serde::{Deserialize, Serialize};

use crate::cross_platform_http_client::{Body, CurrentPlatformClient, HttpClient};
use crate::error::WildlandHttpClientError;
use crate::response_handler::check_status_code;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfirmTokenReq {
    pub session_id: String,
    pub email: String,
    pub verification_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetStorageReq {
    pub session_id: Option<String>,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetStorageRes {
    pub credentials: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Clone)]
pub struct EvsClient {
    http_client: Rc<dyn HttpClient>,
    base_url: String,
}

impl EvsClient {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(base_url: String) -> Self {
        let http_client = Rc::new(CurrentPlatformClient {});

        Self {
            http_client,
            base_url,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn confirm_token(&self, request: ConfirmTokenReq) -> Result<(), WildlandHttpClientError> {
        let request = http::Request::put(format!("{}/confirm_token", self.base_url))
            .body(Body::json(request))?;
        let response = self.http_client.send(request)?;
        check_status_code(response)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_storage(
        &self,
        request: GetStorageReq,
    ) -> Result<GetStorageRes, WildlandHttpClientError> {
        let request = http::Request::put(format!("{}/get_storage", self.base_url))
            .body(Body::json(request))?;

        let response = self.http_client.send(request)?;
        check_status_code(response)?
            .map(|body| serde_json::from_slice(&body))
            .into_body()
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_platform_http_client::MockHttpClient;
    use crate::evs::constants::test_utilities::{CREDENTIALS, EMAIL, VERIFICATION_TOKEN};

    #[test]
    fn should_confirm_token() {
        let mut http_client = Box::new(MockHttpClient::new());

        let request = ConfirmTokenReq {
            email: EMAIL.into(),
            verification_token: VERIFICATION_TOKEN.into(),
            session_id: "some uuid".to_string(),
        };

        let http_request = http::Request::put("/confirm_token")
            .body(Body::json(request.clone()))
            .unwrap();

        http_client
            .as_mut()
            .expect_send()
            .withf(move |request| {
                request.method() == http_request.method()
                    && request.uri() == http_request.uri()
                    && request.headers() == http_request.headers()
                    && request.body() == http_request.body()
            })
            .times(1)
            .returning(|_| {
                Ok(http::Response::builder()
                    .status(200)
                    .body(Vec::default())
                    .unwrap())
            });

        let response = EvsClient {
            http_client: Rc::from(http_client as Box<_>),
            base_url: "".into(),
        }
        .confirm_token(request);

        assert!(response.is_ok());
    }

    #[test]
    fn should_get_storage() {
        let mut http_client = Box::new(MockHttpClient::new());

        let request = GetStorageReq {
            email: EMAIL.into(),
            session_id: Some("some uuid".to_string()),
        };

        let http_request = http::Request::put("/get_storage")
            .body(Body::json(request.clone()))
            .unwrap();

        http_client
            .as_mut()
            .expect_send()
            .withf(move |request| {
                request.method() == http_request.method()
                    && request.uri() == http_request.uri()
                    && request.headers() == http_request.headers()
                    && request.body() == http_request.body()
            })
            .times(1)
            .returning(|_| {
                Ok(http::Response::builder()
                    .status(200)
                    .body(
                        serde_json::to_vec(&serde_json::json!({ "credentials": CREDENTIALS }))
                            .unwrap(),
                    )
                    .unwrap())
            });

        let response = EvsClient {
            http_client: Rc::from(http_client as Box<_>),
            base_url: "".into(),
        }
        .get_storage(request)
        .unwrap();
        assert_eq!(response.credentials.unwrap(), CREDENTIALS);
    }
}
