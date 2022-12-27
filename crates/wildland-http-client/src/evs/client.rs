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
use serde_json::json;

use crate::cross_platform_http_client::{CurrentPlatformClient, HttpClient};
use crate::error::WildlandHttpClientError;
use crate::response_handler::check_status_code;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmTokenReq {
    pub session_id: String,
    pub email: String,
    pub verification_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub(crate) http_client: Rc<dyn HttpClient>,
}

impl EvsClient {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(base_url: &str) -> Self {
        let http_client = Rc::new(CurrentPlatformClient {
            base_url: base_url.into(),
        });

        Self { http_client }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn confirm_token(&self, request: ConfirmTokenReq) -> Result<(), WildlandHttpClientError> {
        let response = self
            .http_client
            .put("confirm_token", Some(json!(request)), None)?;
        check_status_code(response)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_storage(
        &self,
        request: GetStorageReq,
    ) -> Result<GetStorageRes, WildlandHttpClientError> {
        let response = self
            .http_client
            .put("get_storage", Some(json!(request)), None)?;
        let response = check_status_code(response)?;
        let json = response.deserialize()?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use serde_json::json;

    use super::*;
    use crate::cross_platform_http_client::{MockHttpClient, Response};
    use crate::evs::constants::test_utilities::{CREDENTIALS, EMAIL, VERIFICATION_TOKEN};

    #[test]
    fn should_confirm_token() {
        let mut http_client = Box::new(MockHttpClient::new());

        let request = ConfirmTokenReq {
            email: EMAIL.into(),
            verification_token: VERIFICATION_TOKEN.into(),
            session_id: "some uuid".to_string(),
        };

        http_client
            .as_mut()
            .expect_put()
            .with(eq("confirm_token"), eq(Some(json!(request))), eq(None))
            .times(1)
            .returning(|_, _, _| {
                Ok(Response {
                    status_code: 200,
                    body: vec![],
                })
            });

        let response = EvsClient {
            http_client: Rc::from(http_client as Box<dyn HttpClient>),
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

        http_client
            .as_mut()
            .expect_put()
            .with(eq("get_storage"), eq(Some(json!(request))), eq(None))
            .times(1)
            .returning(|_, _, _| {
                Ok(Response {
                    status_code: 200,
                    body: serde_json::to_vec(&json!({ "credentials": CREDENTIALS })).unwrap(),
                })
            });

        let response = EvsClient {
            http_client: Rc::from(http_client as Box<dyn HttpClient>),
        }
        .get_storage(request)
        .unwrap();
        assert_eq!(response.credentials.unwrap(), CREDENTIALS);
    }
}
