use anyhow::Context;

use super::{Body, HttpClient, HttpError, HttpResult, Request};

#[derive(Clone, Default)]
pub struct MinreqHttpClient {}

impl HttpClient for MinreqHttpClient {
    fn send(&self, request: Request) -> HttpResult {
        let method = match request.method().as_str() {
            "GET" => minreq::Method::Get,
            "POST" => minreq::Method::Post,
            "PUT" => minreq::Method::Put,
            "DELETE" => minreq::Method::Delete,
            "HEAD" => minreq::Method::Head,
            "OPTIONS" => minreq::Method::Options,
            "CONNECT" => minreq::Method::Connect,
            "PATCH" => minreq::Method::Patch,
            "TRACE" => minreq::Method::Trace,
            v => minreq::Method::Custom(v.into()),
        };

        let mut req = minreq::Request::new(method, request.uri().to_string());

        for (key, val) in request.headers().iter() {
            req = req.with_header(
                key.as_str(),
                val.to_str()
                    .context("Invalid header value")
                    .map_err(HttpError::user)?,
            );
        }

        req = match request.into_body() {
            Body::Json(v) => req
                .with_json(&v)
                .context("Invalid json body")
                .map_err(HttpError::user)?,
            Body::Raw(v) => req.with_body(v),
        };

        let resp = req
            .send()
            .context("HTTP request failed")
            .map_err(HttpError::io)?;
        let mut builder = http::Response::builder().status(resp.status_code as u16);

        for (k, v) in resp.headers.iter() {
            builder = builder.header(k, v);
        }

        builder
            .body(resp.into_bytes())
            .context("Cant build HTTP responce")
            .map_err(HttpError::other)
    }
}
