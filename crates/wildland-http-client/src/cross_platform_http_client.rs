use std::collections::HashMap;
use std::rc::Rc;

use serde::Deserialize;
use thiserror::Error;

#[cfg(target_os = "emscripten")]
mod emscripten_http_client;

#[cfg(target_os = "emscripten")]
pub(crate) use emscripten_http_client::EmscriptenHttpClient as CurrentPlatformClient;

#[cfg(not(target_os = "emscripten"))]
mod minreq_http_client;

#[cfg(not(target_os = "emscripten"))]
pub(crate) use minreq_http_client::MinreqHttpClient as CurrentPlatformClient;

#[derive(Error, Debug, Clone)]
pub enum HttpError {
    #[error("{0}")]
    Generic(String),
    #[error("{0}")]
    InvalidResponseUTF8Body(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    InvalidResponseJsonBody(#[from] Rc<serde_json::Error>),
}

pub(crate) struct Response {
    pub(crate) status_code: i32,
    pub(crate) body: Vec<u8>,
}

impl Response {
    pub fn to_string(&self) -> Result<String, HttpError> {
        Ok(String::from_utf8(self.body.clone())?)
    }

    pub fn deserialize<'a, T>(&'a self) -> Result<T, HttpError>
    where
        T: Deserialize<'a>,
    {
        Ok(serde_json::from_slice(&self.body).map_err(Rc::new)?)
    }
}

pub(crate) type HttpResult = Result<Response, HttpError>;

#[cfg_attr(test, mockall::automock)]
pub(crate) trait HttpClient {
    fn post(
        &self,
        url: &str,
        json: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> HttpResult;
    fn put(
        &self,
        url: &str,
        json: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
    ) -> HttpResult;
}
