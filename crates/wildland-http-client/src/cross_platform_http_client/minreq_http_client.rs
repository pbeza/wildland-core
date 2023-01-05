use super::{HttpClient, HttpError, HttpResult, Request, Response};

pub(crate) struct MinreqHttpClient {
    pub(crate) base_url: String,
}

impl HttpClient for MinreqHttpClient {
    fn post(&self, request: Request) -> HttpResult {
        let url = format!("{}{}", self.base_url, request.url);
        let mut req = minreq::post(url);
        if let Some(json) = request.json {
            req = req
                .with_json(&json)
                .map_err(|err| HttpError::Generic(err.to_string()))?;
        }

        for (key, val) in request.headers.into_iter() {
            req = req.with_header(key, val);
        }

        let resp = req
            .send()
            .map_err(|err| HttpError::Generic(err.to_string()))?;
        Ok(Response {
            status_code: resp.status_code,
            body: resp.into_bytes(),
        })
    }

    fn put(&self, request: Request) -> HttpResult {
        let url = format!("{}{}", self.base_url, request.url);
        let mut req = minreq::put(url);
        if let Some(json) = request.json {
            req = req
                .with_json(&json)
                .map_err(|err| HttpError::Generic(err.to_string()))?;
        }

        for (key, val) in request.headers.into_iter() {
            req = req.with_header(key, val);
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
