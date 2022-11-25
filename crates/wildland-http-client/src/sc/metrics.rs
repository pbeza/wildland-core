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

use crate::sc::constants::WILDLAND_SIGNATURE_HEADER;
use minreq::{Error, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMetricsReq {
    #[serde(rename(serialize = "credentialID"))]
    pub credential_id: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMetricsRes {
    #[serde(rename(deserialize = "credentialID"))]
    pub credentials_id: String,

    #[serde(rename(deserialize = "usageCred"))]
    pub usage_cred: UsageReq,

    #[serde(rename(deserialize = "usageStorage"))]
    pub usage_storage: UsageReq,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageReq {
    pub rx: i64,
    pub tx: i64,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct SCMetricsClient {
    pub(crate) base_url: String,
}

impl SCMetricsClient {
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn request_metrics(
        &self,
        request: RequestMetricsReq,
        signature: &str,
    ) -> Result<Response, Error> {
        let url = format!("{}/metrics", self.base_url);
        minreq::post(url)
            .with_header(WILDLAND_SIGNATURE_HEADER, signature)
            .with_json(&request)?
            .send()
    }
}

#[cfg(test)]
mod tests {
    use crate::sc::constants::test_utilities::{CREDENTIALS_ID, SIGNATURE, TIMESTAMP};
    use mockito::{mock, server_url};
    use serde_json::json;

    use super::*;

    fn client() -> SCMetricsClient {
        SCMetricsClient {
            base_url: server_url(),
        }
    }

    #[test]
    fn should_get_storage_metrics() {
        let request = RequestMetricsReq {
            credential_id: CREDENTIALS_ID.to_string(),
            timestamp: TIMESTAMP.to_string(),
        };

        let m = mock("POST", "/metrics")
            .with_body(
                json!({
                    "credentialID" : CREDENTIALS_ID,
                    "usageCred" : {
                        "rx" : 433_i64,
                        "tx" : 523_i64
                    },
                    "usageStorage" : {
                        "rx" : 433_i64,
                        "tx" : 523_i64
                    }
                })
                .to_string(),
            )
            .create();

        let response = client()
            .request_metrics(request, SIGNATURE)
            .unwrap()
            .json::<RequestMetricsRes>()
            .unwrap();

        m.assert();
        assert_eq!(response.credentials_id, CREDENTIALS_ID);
        assert_eq!(response.usage_cred.rx, 433);
        assert_eq!(response.usage_cred.tx, 523);
        assert_eq!(response.usage_storage.rx, 433);
        assert_eq!(response.usage_cred.tx, 523);
    }
}
