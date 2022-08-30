use crate::constants::WILDLAND_SIGNATURE_HEADER;
use reqwest::{Client, Error, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureRequestReq {
    #[serde(rename(serialize = "credentialID"))]
    pub credential_id: String,
    pub timestamp: String,
    #[serde(rename(serialize = "storageMethod"))]
    pub storage_method: String,
    #[serde(rename(serialize = "storagePath"))]
    pub storage_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureRequestRes {
    pub message: String,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct SCSignatureClient {
    pub(crate) base_url: String,
    pub(crate) client: Client,
}

impl SCSignatureClient {
    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub(crate) async fn signature_request(
        &self,
        request: SignatureRequestReq,
        signature: &str,
    ) -> Result<Response, Error> {
        let url = format!("{}/signature/request", self.base_url);
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
    use crate::constants::test_utilities::{CREDENTIALS_ID, MESSAGE, SIGNATURE, TIMESTAMP};
    use mockito::{mock, server_url};
    use serde_json::json;

    use super::*;

    fn client() -> SCSignatureClient {
        SCSignatureClient {
            base_url: server_url(),
            client: Client::new(),
        }
    }

    #[tokio::test]
    async fn should_receive_signed_url() {
        // given
        let request = SignatureRequestReq {
            credential_id: CREDENTIALS_ID.into(),
            storage_method: "put".into(),
            storage_path: "/".into(),
            timestamp: TIMESTAMP.into(),
        };

        let m = mock("POST", "/signature/request")
            .with_body(json!({ "message": MESSAGE }).to_string())
            .create();

        // when
        let response = client()
            .signature_request(request, SIGNATURE)
            .await
            .unwrap()
            .json::<SignatureRequestRes>()
            .await
            .unwrap();

        // then
        m.assert();
        assert_eq!(response.message, MESSAGE);
    }
}
