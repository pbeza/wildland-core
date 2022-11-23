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

use crate::{error::WildlandHttpClientError, response_handler::check_status_code};
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
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn confirm_token(&self, request: ConfirmTokenReq) -> Result<(), WildlandHttpClientError> {
        let url = format!("{}/confirm_token", self.base_url);
        let response = minreq::put(url)
            .with_json(&request)
            .map_err(|e| WildlandHttpClientError::HttpLibError(Rc::new(e)))?
            .send()
            .map_err(Rc::new)?;
        check_status_code(response)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
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
        let response = check_status_code(response)?;
        match response.status_code {
            200 => Ok(response.json().map_err(Rc::new)?),
            // Status 2xx without body
            _ => Ok(GetStorageRes {
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
