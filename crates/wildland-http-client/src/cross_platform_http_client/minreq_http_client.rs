use std::collections::HashMap;

use super::{HttpClient, HttpError, HttpResult, Response};

pub(crate) struct MinreqHttpClient {
    pub(crate) base_url: String,
}

impl HttpClient for MinreqHttpClient {
    fn post(
        &self,
        url: &str,
        json: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> HttpResult {
        let url = format!("{}/{}", self.base_url, url);
        let mut req = minreq::post(url);
        if let Some(json) = json {
            req = req
                .with_json(&json)
                .map_err(|err| HttpError::Generic(err.to_string()))?;
        }

        if let Some(headers) = headers {
            for (key, val) in headers.into_iter() {
                req = req.with_header(key, val);
            }
        }

        let resp = req
            .send()
            .map_err(|err| HttpError::Generic(err.to_string()))?;
        Ok(Response {
            status_code: resp.status_code,
            body: resp.into_bytes(),
        })
    }

    fn put(
        &self,
        url: &str,
        json: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> HttpResult {
        let url = format!("{}/{}", self.base_url, url);
        let mut req = minreq::put(url);
        if let Some(json) = json {
            req = req
                .with_json(&json)
                .map_err(|err| HttpError::Generic(err.to_string()))?;
        }

        if let Some(headers) = headers {
            for (key, val) in headers.into_iter() {
                req = req.with_header(key, val);
            }
        }

        let resp = req
            .send()
            .map_err(|err| HttpError::Generic(err.to_string()))?;
        Ok(Response {
            status_code: resp.status_code,
            body: resp.into_bytes(),
        })
    }
}
