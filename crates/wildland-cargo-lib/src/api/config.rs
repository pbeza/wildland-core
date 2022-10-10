//! This module consists of types and functions for creating configuration of [`super::CargoLib`].
//!
//! Configuration may be represented by a JSON like:
//! ```
//! # use wildland_cargo_lib::api::config::parse_config;
//! #
//! let config_json = r#"
//!     {
//!         "log_level": "debug",
//!         "log_file": "optional file path - it turns on file logging if provided"
//!     }
//! "#;
//!
//! let _  = parse_config(config_json.as_bytes().to_vec()).unwrap();
//! ```
//!
//! It can be also provided via type implementing [`CargoCfgProvider`].

use serde::Deserialize;
use thiserror::Error;

use crate::errors::{SingleErrVariantResult, SingleVariantError};

pub trait CargoCfgProvider {
    fn get_log_level(&self) -> String;
    fn get_log_file(&self) -> Option<String>;
}

#[derive(PartialEq, Eq, Error, Debug, Clone)]
#[error("Config parse error: {0}")]
pub struct ParseConfigError(pub String);

/// Structure representing configuration for [`super::CargoLib`] initialization.
///
/// It can be created outside of Rust in the following ways:
/// - by implementing [`CargoCfgProvider`] and calling [`collect_config`] function with that type as an argument
/// - calling [`parse_config`]
///
#[derive(Debug, Deserialize, Clone)]
pub struct CargoConfig {
    pub log_level: String,
    pub log_file: Option<String>,
}

impl CargoCfgProvider for CargoConfig {
    fn get_log_level(&self) -> String {
        self.log_level.clone()
    }

    fn get_log_file(&self) -> Option<String> {
        self.log_file.clone()
    }
}

/// Uses an implementation of [`CargoCfgProvider`] to collect a configuration storing structure ([`CargoConfig`])
/// which then can be passed to [`super::cargo_lib::create_cargo_lib`] in order to instantiate main API object ([`super::CargoLib`])
///
pub fn collect_config(config_provider: &'static dyn CargoCfgProvider) -> CargoConfig {
    CargoConfig {
        log_level: config_provider.get_log_level(),
        log_file: config_provider.get_log_file(),
    }
}

/// Parses bytes representing JSON formatted configuration of [`super::CargoLib`]
///
pub fn parse_config(raw_content: Vec<u8>) -> SingleErrVariantResult<CargoConfig, ParseConfigError> {
    let parsed: CargoConfig = serde_json::from_slice(&raw_content)
        .map_err(|e| SingleVariantError::Failure(ParseConfigError(e.to_string())))?;
    println!("Parsed config: {parsed:?}");
    Ok(parsed)
}
