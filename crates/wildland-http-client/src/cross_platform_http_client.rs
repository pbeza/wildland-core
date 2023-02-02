use std::rc::Rc;

use serde::Serialize;
use serde_json::json;
use thiserror::Error;

#[cfg(target_os = "emscripten")]
mod emscripten_http_client;

#[cfg(target_os = "emscripten")]
pub use emscripten_http_client::EmscriptenHttpClient as CurrentPlatformClient;

#[cfg(not(target_os = "emscripten"))]
mod minreq_http_client;

#[cfg(not(target_os = "emscripten"))]
pub use minreq_http_client::MinreqHttpClient as CurrentPlatformClient;

#[derive(Error, Debug, Clone)]
#[repr(C)]
pub enum HttpError {
    #[error("User error: {0}")]
    User(Rc<anyhow::Error>),
    #[error("Io error: {0}")]
    Io(Rc<anyhow::Error>),
    #[error("Other error: {0}")]
    Other(Rc<anyhow::Error>),
}

impl HttpError {
    pub fn user<T>(err: T) -> Self
    where
        T: Into<anyhow::Error>,
    {
        Self::User(Rc::new(err.into()))
    }

    pub fn io<T>(err: T) -> Self
    where
        T: Into<anyhow::Error>,
    {
        Self::Io(Rc::new(err.into()))
    }

    pub fn other<T>(err: T) -> Self
    where
        T: Into<anyhow::Error>,
    {
        Self::Other(Rc::new(err.into()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    Json(serde_json::Value),
    Raw(Vec<u8>),
}

impl Body {
    pub fn json<T>(val: T) -> Self
    where
        T: Serialize,
    {
        Self::Json(json!(val))
    }

    pub fn raw(val: Vec<u8>) -> Self {
        Self::Raw(val)
    }

    pub fn empty() -> Self {
        Self::raw(Vec::new())
    }
}

pub type Request = http::Request<Body>;
pub type Response = http::Response<Vec<u8>>;

pub type HttpResult = Result<Response, HttpError>;

#[cfg_attr(test, mockall::automock)]
pub trait HttpClient {
    fn send(&self, request: Request) -> HttpResult;
}
