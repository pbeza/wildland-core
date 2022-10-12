use crate::sc::constants::WILDLAND_SIGNATURE_HEADER;
use minreq::{Error, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCredentialsReq {
    #[serde(rename(serialize = "credentialID"))]
    pub credential_id: String,
    pub timestamp: String,
    #[serde(rename(serialize = "credPermission"))]
    pub cred_permission: String,
    #[serde(rename(serialize = "fsPermission"))]
    pub fs_permission: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCredentialsRes {
    #[serde(rename(deserialize = "credentialID"))]
    pub credentials_id: String,
    #[serde(rename(deserialize = "credentialSecret"))]
    pub credentials_secret: String,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct SCCredentialsClient {
    pub(crate) base_url: String,
}

impl SCCredentialsClient {
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub(crate) fn create_credentials(
        &self,
        request: CreateCredentialsReq,
        signature: &str,
    ) -> Result<Response, Error> {
        let url = format!("{}/credential/create", self.base_url);
        minreq::post(url)
            .with_header(WILDLAND_SIGNATURE_HEADER, signature)
            .with_json(&request)?
            .send()
    }
}

#[cfg(test)]
mod tests {
    use crate::sc::constants::test_utilities::{
        CREDENTIALS_ID, CREDENTIALS_SECRET, SIGNATURE, TIMESTAMP,
    };
    use mockito::{mock, server_url};
    use serde_json::json;

    use super::*;

    #[tracing::instrument]
    fn client() -> SCCredentialsClient {
        SCCredentialsClient {
            base_url: server_url(),
        }
    }

    #[tokio::test]
    async fn storage_credentials_can_be_created() {
        // given
        let request = CreateCredentialsReq {
            credential_id: CREDENTIALS_ID.to_string(),
            timestamp: TIMESTAMP.to_string(),
            cred_permission: "CRSR".to_string(),
            fs_permission: "write".to_string(),
            path: "/".to_string(),
        };

        let mock = mock("POST", "/credential/create")
            .with_status(201)
            .with_body(
                json!({
                    "credentialID" : CREDENTIALS_ID,
                    "credentialSecret" : CREDENTIALS_SECRET
                })
                .to_string(),
            )
            .create();

        // when
        let response = client()
            .create_credentials(request, SIGNATURE)
            .unwrap()
            .json::<CreateCredentialsRes>()
            .unwrap();

        // then
        mock.assert();
        assert_eq!(response.credentials_id, CREDENTIALS_ID);
        assert_eq!(response.credentials_secret, CREDENTIALS_SECRET);
    }
}
