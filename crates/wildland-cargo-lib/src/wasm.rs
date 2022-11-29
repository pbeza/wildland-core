use crate::api::config::*;
use async_trait::async_trait;
use futures::{future::FutureExt, TryFutureExt};
use std::{future::Future, path::PathBuf, str::FromStr};
use tracing::Level;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use wildland_corex::LssError;

#[wasm_bindgen]
extern "C" {
    pub type JsConfigProvider;

    #[wasm_bindgen(structural, method)]
    pub fn get_use_logger(this: &JsConfigProvider) -> bool;

    #[wasm_bindgen(structural, getter, method)]
    fn log_level(this: &JsConfigProvider) -> String;

    #[wasm_bindgen(structural, method)]
    fn get_log_file_path(this: &JsConfigProvider) -> Option<String>;
}

#[wasm_bindgen]
pub fn collect_cfg(js_cfg: &JsConfigProvider) -> CargoConfig {
    let b = js_cfg.get_use_logger();
    let l = js_cfg.log_level();
    let lfp = js_cfg.get_log_file_path();

    // this function could construct rust ConfigProvider, Config directly
    CargoConfig {
        fsa_config: FoundationStorageApiConfig {
            evs_url: "".to_string(),
            sc_url: "".to_string(),
        },
        logger_config: LoggerConfig {
            use_logger: js_cfg.get_use_logger(),
            log_level: Level::from_str(js_cfg.log_level().as_str()).unwrap(),
            log_use_ansi: true,
            log_file_enabled: true,
            log_file_path: PathBuf::from(
                js_cfg
                    .get_log_file_path()
                    .unwrap_or("some path".to_string()),
            ),
            log_file_rotate_directory: PathBuf::from("dafs"),
            oslog_category: None,
            oslog_subsystem: None,
        },
    }
}

#[wasm_bindgen]
extern "C" {
    pub type JsLss;

    #[wasm_bindgen(typescript_type = "Array<string>")]
    pub type StringArray;

    #[wasm_bindgen(structural, method, catch, js_name = insert)]
    pub fn js_insert(this: &JsLss, key: String, value: String) -> Result<Option<String>, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = get)]
    pub fn js_get(this: &JsLss, key: String) -> Result<Option<String>, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = contains_key)]
    pub fn js_contains_key(this: &JsLss, key: String) -> Result<bool, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = keys)]
    pub fn js_keys(this: &JsLss) -> Result<StringArray, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = keys_starting_with)]
    pub fn js_keys_starting_with(this: &JsLss, prefix: String) -> Result<StringArray, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = remove)]
    pub fn js_remove(this: &JsLss, key: String) -> Result<Option<String>, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = len)]
    pub fn js_len(this: &JsLss) -> Result<usize, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = is_empty)]
    pub fn js_is_empty(this: &JsLss) -> Result<bool, LssError>;
}

pub struct WasmRedisClient;
impl RedisClientTrait for WasmRedisClient {
    fn set(&self, key: &str, val: &str) -> Box<dyn Future<Output = ()>> {
        let mut req_options = RequestInit::new();
        req_options.method("GET");
        req_options.mode(RequestMode::Cors);
        let req = Request::new_with_str_and_init(
            &format!("http://127.0.0.1:7379/SET/forest-{}/{}", key, val),
            &req_options,
        )
        .unwrap();
        let window = web_sys::window().unwrap();
        let resp_value_future = JsFuture::from(window.fetch_with_request(&req));
        let result_future = resp_value_future.then(|resp_res| async {
            let resp = resp_res.unwrap();
        });
        Box::new(result_future)
    }
}

pub struct TcpRedisClient;
impl RedisClientTrait for TcpRedisClient {
    fn set(&self, key: &str, val: &str) -> Box<dyn Future<Output = ()>> {
        Box::new(async { () })
    }
}

pub trait RedisClientTrait {
    fn set(&self, key: &str, val: &str) -> Box<dyn Future<Output = ()>>;
}

pub struct RedisClient(Box<dyn RedisClientTrait>);

impl RedisClient {
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        return Self(Box::new(WasmRedisClient));
        #[cfg(not(target_arch = "wasm32"))]
        panic!("Unsupported arch for Redis Client")
    }
}
