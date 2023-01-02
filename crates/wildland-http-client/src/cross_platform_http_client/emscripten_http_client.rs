mod emscripten {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::sync::mpsc;

use super::{HttpClient, HttpError, HttpResult, Request, Response};

struct Fetch {
    fetch_attr: emscripten::emscripten_fetch_attr_t,
    json: CString,
    _headers: Vec<CString>,
    headers_ptrs: Vec<*const i8>,
    tx: mpsc::Sender<HttpResult>,
    rx: mpsc::Receiver<HttpResult>,
    fetch_handler: Option<*mut emscripten::emscripten_fetch_t>,
}

extern "C" fn onsuccess(result: *mut emscripten::emscripten_fetch_t) {
    unsafe {
        let fetch = (*result).userData as *mut Fetch;

        let resp = Response {
            status_code: (*result).status.into(),
            body: std::slice::from_raw_parts(
                (*result).data as *const u8,
                (*result).numBytes as usize,
            )
            .to_vec(),
        };

        let _ = (*fetch).tx.send(Ok(resp));
    };
}

extern "C" fn onerror(result: *mut emscripten::emscripten_fetch_t) {
    unsafe {
        let fetch = (*result).userData as *mut Fetch;

        let status_code = (*result).status;
        let status_text = CStr::from_ptr((*result).statusText.as_ptr()).to_string_lossy();
        let url = CStr::from_ptr((*result).url).to_string_lossy();

        let _ = (*fetch).tx.send(Err(HttpError::Generic(format!(
            "FETCH request to {url} failed with {status_code} status code: {status_text}"
        ))));
    }
}

impl Fetch {
    pub fn new(
        method: &str,
        json: Option<serde_json::Value>,
        headers: HashMap<String, String>,
    ) -> Result<Box<Self>, HttpError> {
        let fetch_attr = unsafe {
            let mut fetch_attr = std::mem::zeroed::<emscripten::emscripten_fetch_attr_t>();
            emscripten::emscripten_fetch_attr_init(&mut fetch_attr);
            fetch_attr
        };

        let cjson = match json {
            Some(val) => {
                CString::new(val.to_string()).map_err(|e| HttpError::Generic(e.to_string()))?
            }
            None => CString::default(),
        };

        let cheaders = headers
            .into_iter()
            .flat_map(|(key, val)| [CString::new(key), CString::new(val)])
            .collect::<Result<_, _>>()
            .map_err(|e| HttpError::Generic(e.to_string()))?;

        let headers_ptrs = cheaders
            .iter()
            .map(|e| e.as_ptr())
            .chain(std::iter::once(std::ptr::null()))
            .collect();

        let (tx, rx) = mpsc::channel();

        let mut fetch = Box::new(Fetch {
            fetch_attr,
            json: cjson,
            headers_ptrs,
            _headers: cheaders,
            tx,
            rx,
            fetch_handler: None,
        });

        for (i, c) in method.chars().enumerate() {
            fetch.fetch_attr.requestMethod[i] = c as i8;
        }

        fetch.fetch_attr.attributes = emscripten::EMSCRIPTEN_FETCH_LOAD_TO_MEMORY
            // Disable interaction with IndexDB to prevent deadlock.
            | emscripten::EMSCRIPTEN_FETCH_REPLACE
            // Async downloading also causes deadlock. This flag also makes the SDK usable only as a web worker.
            | emscripten::EMSCRIPTEN_FETCH_SYNCHRONOUS;

        fetch.fetch_attr.requestHeaders = fetch.headers_ptrs.as_ptr();
        fetch.fetch_attr.requestData = fetch.json.as_ptr();
        fetch.fetch_attr.requestDataSize = fetch.json.to_bytes().len();
        fetch.fetch_attr.userData = &mut *fetch as *mut _ as *mut c_void;
        fetch.fetch_attr.onsuccess = Some(onsuccess);
        fetch.fetch_attr.onerror = Some(onerror);

        Ok(fetch)
    }

    fn send(mut self: Box<Self>, url: &str) -> HttpResult {
        let url = CString::new(url).map_err(|e| HttpError::Generic(e.to_string()))?;
        unsafe {
            self.fetch_handler = Some(emscripten::emscripten_fetch(
                &mut self.fetch_attr,
                url.as_ptr(),
            ));
        }
        self.rx
            .recv()
            .map_err(|e| HttpError::Generic(e.to_string()))?
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

pub(crate) struct EmscriptenHttpClient {
    pub(crate) base_url: String,
}

impl HttpClient for EmscriptenHttpClient {
    fn post(&self, request: Request) -> HttpResult {
        let url = format!("{}{}", self.base_url, request.url);
        let fetch = Fetch::new("POST", request.json, request.headers)?;
        fetch.send(&url)
    }

    fn put(&self, request: Request) -> HttpResult {
        let url = format!("{}{}", self.base_url, request.url);
        let fetch = Fetch::new("PUT", request.json, request.headers)?;
        fetch.send(&url)
    }
}
