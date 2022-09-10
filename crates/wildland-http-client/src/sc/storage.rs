use reqwest::{Client, Error, Response};
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
    pub(crate) client: Client,
}

impl SCStorageClient {
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub(crate) async fn create_storage(&self) -> Result<Response, Error> {
        let url = format!("{}/storage/create", self.base_url);
        self.client.post(url).send().await
    }
}

#[cfg(test)]
mod tests {
    use mockito::{mock, server_url};
    use serde_json::json;
    use crate::sc::constants::test_utilities::{CREDENTIALS_ID, CREDENTIALS_SECRET, STORAGE_ID};

    use super::*;

    fn client() -> SCStorageClient {
        SCStorageClient {
            base_url: server_url(),
            client: Client::new(),
        }
    }

    #[tokio::test]
    async fn storage_can_be_created() {
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
            .await
            .unwrap()
            .json::<CreateStorageRes>()
            .await
            .unwrap();

        m.assert();
        assert_eq!(response.storage_id, STORAGE_ID);
        assert_eq!(response.credentials_id, CREDENTIALS_ID);
        assert_eq!(response.credentials_secret, CREDENTIALS_SECRET);
    }
}
