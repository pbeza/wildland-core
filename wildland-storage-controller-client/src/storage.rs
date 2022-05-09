use reqwest::{Client, Error, Response};

#[derive(Clone, Default)]
pub(crate) struct SCStorageClient {
    pub(crate) base_url: String,
    pub(crate) client: Client,
}

impl SCStorageClient {
    pub(crate) async fn create_storage(&self) -> Result<Response, Error> {
        let url = format!("{}/storage/create", self.base_url);
        self.client.post(url).send().await
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::test_utilities::{CREDENTIALS_ID, CREDENTIALS_SECRET};
    use mockito::{mock, server_url};
    use serde_json::json;

    use crate::CreateCredentialsRes;

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
            .json::<CreateCredentialsRes>()
            .await
            .unwrap();

        m.assert();
        assert_eq!(response.credentials_id, CREDENTIALS_ID);
        assert_eq!(response.credentials_secret, CREDENTIALS_SECRET);
    }
}
