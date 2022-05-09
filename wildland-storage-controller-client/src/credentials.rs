use crate::constants::WILDLAND_SIGNATURE_HEADER;
use reqwest::{Client, Error, Response};
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

#[derive(Clone, Default)]
pub(crate) struct SCCredentialsClient {
    pub(crate) base_url: String,
    pub(crate) client: Client,
}

impl SCCredentialsClient {
    pub(crate) async fn create_credentials(
        &self,
        request: CreateCredentialsReq,
        signature: &str,
    ) -> Result<Response, Error> {
        let url = format!("{}/credential/create", self.base_url);
        self.client
            .post(url)
            .header(WILDLAND_SIGNATURE_HEADER, signature)
            .json(&request)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::test_utilities::{
        CREDENTIALS_ID, CREDENTIALS_SECRET, SIGNATURE, TIMESTAMP,
    };
    use mockito::{mock, server_url};
    use serde_json::json;

    use crate::CreateCredentialsRes;

    use super::*;

    fn client() -> SCCredentialsClient {
        SCCredentialsClient {
            base_url: server_url(),
            client: Client::new(),
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
            .await
            .unwrap()
            .json::<CreateCredentialsRes>()
            .await
            .unwrap();

        // then
        mock.assert();
        assert_eq!(response.credentials_id, CREDENTIALS_ID);
        assert_eq!(response.credentials_secret, CREDENTIALS_SECRET);
    }
}
