use crate::constants::WILDLAND_SIGNATURE_HEADER;
use reqwest::{Client, Error, Response};
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

#[derive(Clone, Default)]
pub(crate) struct SCMetricsClient {
    pub(crate) base_url: String,
    pub(crate) client: Client,
}

impl SCMetricsClient {
    pub(crate) async fn request_metrics(
        &self,
        request: RequestMetricsReq,
        signature: &str,
    ) -> Result<Response, Error> {
        let url = format!("{}/metrics", self.base_url);
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
    use crate::constants::test_utilities::{CREDENTIALS_ID, SIGNATURE, TIMESTAMP};
    use mockito::{mock, server_url};
    use serde_json::json;

    use super::*;

    fn client() -> SCMetricsClient {
        SCMetricsClient {
            base_url: server_url(),
            client: Client::new(),
        }
    }

    #[tokio::test]
    async fn should_get_storage_metrics() {
        let request = RequestMetricsReq {
            credential_id: CREDENTIALS_ID.to_string(),
            timestamp: TIMESTAMP.to_string(),
        };

        let m = mock("POST", "/metrics")
            .with_body(
                json!({
                    "credentialID" : CREDENTIALS_ID,
                    "usageCred" : {
                        "rx" : 433 as i64,
                        "tx" : 523 as i64
                    },
                    "usageStorage" : {
                        "rx" : 433 as i64,
                        "tx" : 523 as i64
                    }
                })
                .to_string(),
            )
            .create();

        let response = client()
            .request_metrics(request, SIGNATURE)
            .await
            .unwrap()
            .json::<RequestMetricsRes>()
            .await
            .unwrap();

        m.assert();
        assert_eq!(response.credentials_id, CREDENTIALS_ID);
        assert_eq!(response.usage_cred.rx, 433);
        assert_eq!(response.usage_cred.tx, 523);
        assert_eq!(response.usage_storage.rx, 433);
        assert_eq!(response.usage_cred.tx, 523);
    }
}
