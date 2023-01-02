use std::collections::HashMap;
use std::rc::Rc;

use serde::Deserialize;
use serde_json::json;
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Request {
    url: String,
    json: Option<serde_json::Value>,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn new<T>(url: T) -> Self
    where
        T: Into<String>,
    {
        Request {
            url: url.into(),
            json: None,
            headers: HashMap::default(),
        }
    }

    pub fn with_json<T>(mut self, json: &T) -> Self
    where
        T: serde::Serialize,
    {
        self.json = Some(json!(json));
        self
    }

    pub fn with_header<T, U>(mut self, key: T, val: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        self.headers.insert(key.into(), val.into());
        self
    }
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait HttpClient {
    fn post(&self, request: Request) -> HttpResult;
    fn put(&self, request: Request) -> HttpResult;
}
