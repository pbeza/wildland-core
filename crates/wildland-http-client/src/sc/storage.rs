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

use minreq::{Error, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStorageRes {
    #[serde(rename(deserialize = "id"))]
    pub storage_id: String,
    #[serde(rename(deserialize = "credentialID"))]
    pub credentials_id: String,
    #[serde(rename(deserialize = "credentialSecret"))]
    pub credentials_secret: String,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct SCStorageClient {
    pub(crate) base_url: String,
}

impl SCStorageClient {
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn create_storage(&self) -> Result<Response, Error> {
        let url = format!("{}/storage/create", self.base_url);
        minreq::post(url).send()
    }
}

#[cfg(test)]
mod tests {
    use crate::sc::constants::test_utilities::{CREDENTIALS_ID, CREDENTIALS_SECRET, STORAGE_ID};
    use mockito::{mock, server_url};
    use serde_json::json;

    use super::*;

    fn client() -> SCStorageClient {
        SCStorageClient {
            base_url: server_url(),
        }
    }

    #[test]
    fn storage_can_be_created() {
        let m = mock("POST", "/storage/create")
            .with_body(
                json!({
                    "id" : STORAGE_ID,
                    "credentialID" : CREDENTIALS_ID,
                    "credentialSecret" : CREDENTIALS_SECRET
                })
                .to_string(),
            )
            .create();

        let response = client()
            .create_storage()
            .unwrap()
            .json::<CreateStorageRes>()
            .unwrap();

        m.assert();
        assert_eq!(response.storage_id, STORAGE_ID);
        assert_eq!(response.credentials_id, CREDENTIALS_ID);
        assert_eq!(response.credentials_secret, CREDENTIALS_SECRET);
    }
}
