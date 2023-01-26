mod emscripten {
    #![allow(non_upper_case_globals)]
    #![allow(clippy::upper_case_acronyms)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::ffi::{c_void, CString};
use std::pin::Pin;
use std::sync::mpsc;

use anyhow::{anyhow, Context};

use super::{Body, HttpClient, HttpError, HttpResult, Request};

struct Fetch {
    fetch_attr: emscripten::emscripten_fetch_attr_t,
    body: Vec<u8>,
    _headers: Vec<CString>,
    headers_ptrs: Vec<*const i8>,
    tx: mpsc::Sender<HttpResult>,
    rx: mpsc::Receiver<HttpResult>,
    fetch_handler: Option<*mut emscripten::emscripten_fetch_t>,
    url: CString,
}

extern "C" fn onsuccess(result: *mut emscripten::emscripten_fetch_t) {
    unsafe {
        let fetch = (*result).userData as *mut Fetch;

        let resp = http::Response::builder()
            .status((*result).status)
            .body(std::slice::from_raw_parts((*result).data as _, (*result).numBytes as _).to_vec())
            .context("Failed to build HTTP response")
            .map_err(HttpError::other);

        let _ = (*fetch).tx.send(resp);
    };
}

extern "C" fn onerror(result: *mut emscripten::emscripten_fetch_t) {
    unsafe {
        let fetch = (*result).userData as *mut Fetch;
        let status_code = (*result).status;

        let resp = if status_code == 0 {
            Err(HttpError::io(anyhow!("Unknown io error")))
        } else {
            http::Response::builder()
                .status((*result).status)
                .body(
                    std::slice::from_raw_parts((*result).data as _, (*result).numBytes as _)
                        .to_vec(),
                )
                .context("Failed to build HTTP response")
                .map_err(HttpError::other)
        };

        let _ = (*fetch).tx.send(resp);
    }
}

impl Fetch {
    pub fn new(request: Request) -> Result<Pin<Box<Self>>, HttpError> {
        let mut fetch_attr = unsafe {
            let mut fetch_attr = std::mem::zeroed::<emscripten::emscripten_fetch_attr_t>();
            emscripten::emscripten_fetch_attr_init(&mut fetch_attr);
            fetch_attr
        };

        let url = CString::new(request.uri().to_string())
            .context("Invalid uri")
            .map_err(HttpError::user)?;

        for (i, c) in request.method().as_str().chars().enumerate() {
            fetch_attr.requestMethod[i] = c as _;
        }

        let mut headers = request
            .headers()
            .iter()
            .flat_map(|(key, val)| {
                [
                    CString::new(key.as_str().to_owned())
                        .context("Invalid header name")
                        .map_err(HttpError::user),
                    val.to_str()
                        .context("Unprintable header value")
                        .map_err(HttpError::user)
                        .and_then(|v| {
                            CString::new(v)
                                .context("Invalid header value")
                                .map_err(HttpError::user)
                        }),
                ]
            })
            .collect::<Result<Vec<_>, _>>()?;

        let body = match request.into_body() {
            Body::Json(v) => {
                headers.push(CString::new("Content-Type").unwrap());
                headers.push(CString::new("application/json; charset=UTF-8").unwrap());

                serde_json::to_vec(&v)
                    .context("Invalid json body")
                    .map_err(HttpError::user)?
            }
            Body::Raw(v) => v,
        };

        let headers_ptrs = headers
            .iter()
            .map(|e| e.as_ptr())
            .chain(std::iter::once(std::ptr::null()))
            .collect();

        let (tx, rx) = mpsc::channel();

        let mut fetch = Box::pin(Fetch {
            fetch_attr,
            body,
            _headers: headers,
            headers_ptrs,
            tx,
            rx,
            fetch_handler: None,
            url,
        });

        fetch.fetch_attr.attributes = emscripten::EMSCRIPTEN_FETCH_LOAD_TO_MEMORY
            // Disable interaction with IndexDB to prevent deadlock.
            | emscripten::EMSCRIPTEN_FETCH_REPLACE
            // Async downloading also causes deadlock. This flag also makes the SDK usable only as a web worker.
            | emscripten::EMSCRIPTEN_FETCH_SYNCHRONOUS;

        fetch.fetch_attr.requestHeaders = fetch.headers_ptrs.as_ptr();
        fetch.fetch_attr.requestData = fetch.body.as_ptr() as _;
        fetch.fetch_attr.requestDataSize = fetch.body.len();
        fetch.fetch_attr.userData = &mut *fetch as *mut _ as *mut c_void;
        fetch.fetch_attr.onsuccess = Some(onsuccess);
        fetch.fetch_attr.onerror = Some(onerror);
        fetch.fetch_attr.timeoutMSecs = 10_000; //10 sec

        Ok(fetch)
    }

    fn send(mut self: Pin<Box<Self>>) -> HttpResult {
        unsafe {
            self.fetch_handler = Some(emscripten::emscripten_fetch(
                &mut self.fetch_attr,
                self.url.as_ptr(),
            ));
        }
        self.rx
            .recv()
            .context("mpsc error")
            .map_err(HttpError::other)?
    }
}

impl Drop for Fetch {
    fn drop(&mut self) {
        if let Some(fetch_handler) = self.fetch_handler.take() {
            unsafe {
                emscripten::emscripten_fetch_close(fetch_handler);
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct EmscriptenHttpClient {}

impl HttpClient for EmscriptenHttpClient {
    fn send(&self, request: Request) -> HttpResult {
        let fetch = Fetch::new(request)?;
        fetch.send()
    }
}
