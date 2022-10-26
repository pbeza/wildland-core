//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! This module consists of types and functions for creating configuration of [`super::CargoLib`].
//!
//! Configuration may be represented by a JSON like:
//! ```
//! # use wildland_cargo_lib::api::config::parse_config;
//! #
//! let config_json = r#"
//!     {
//!         "log_level": "debug",
//!         "log_file": "optional file path - it turns on file logging if provided",
//!         "evs_url": "some_url",
//!         "sc_url": "some_url"
//!     }
//! "#;
//!
//! let _  = parse_config(config_json.as_bytes().to_vec()).unwrap();
//! ```
//!
//! It can be also provided via type implementing [`CargoCfgProvider`].

use std::str::FromStr;

use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer,
};
use thiserror::Error;
use tracing::Level;

use crate::errors::{SingleErrVariantResult, SingleVariantError};

pub trait CargoCfgProvider {
    /// Must return one of (case-insensitive):
    /// - "error"
    /// - "warn"
    /// - "info"
    /// - "debug"
    /// - "trace"
    /// or number equivalent:
    /// - "1" - error
    /// - "2" - warn
    /// - "3" - info
    /// - "4" - debug
    /// - "5" - trace
    fn get_log_level(&self) -> String;

    fn get_log_file(&self) -> Option<String>;
    fn get_evs_url(&self) -> String;
    fn get_sc_url(&self) -> String;
}

#[derive(PartialEq, Eq, Error, Debug, Clone)]
#[error("Config parse error: {0}")]
pub struct ParseConfigError(pub String);

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct FoundationStorageApiConfig {
    pub evs_url: String,
    pub sc_url: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct LoggerConfig {
    #[serde(deserialize_with = "log_level_deserialize")]
    pub log_level: Level,
    pub log_file: Option<String>,
}

fn log_level_deserialize<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Level::from_str(s.as_ref()).map_err(|_e| {
        Error::invalid_value(Unexpected::Str(&s), &"trace | debug | info | warn | error")
    })
}

/// Structure representing configuration for [`super::CargoLib`] initialization.
///
/// It can be created outside of Rust in the following ways:
/// - by implementing [`CargoCfgProvider`] and calling [`collect_config`] function with that type as an argument
/// - calling [`parse_config`]
///
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct CargoConfig {
    #[serde(flatten)]
    pub fsa_config: FoundationStorageApiConfig,
    #[serde(flatten)]
    pub logger_config: LoggerConfig,
}

/// Uses an implementation of [`CargoCfgProvider`] to collect a configuration storing structure ([`CargoConfig`])
/// which then can be passed to [`super::cargo_lib::create_cargo_lib`] in order to instantiate main API object ([`super::CargoLib`])
///
pub fn collect_config(
    config_provider: &'static dyn CargoCfgProvider,
) -> SingleErrVariantResult<CargoConfig, ParseConfigError> {
    Ok(CargoConfig {
        logger_config: LoggerConfig {
            log_level: Level::from_str(config_provider.get_log_level().as_str())
                .map_err(|e| SingleVariantError::Failure(ParseConfigError(e.to_string())))?,
            log_file: config_provider.get_log_file(),
        },
        fsa_config: FoundationStorageApiConfig {
            evs_url: config_provider.get_evs_url(),
            sc_url: config_provider.get_sc_url(),
        },
    })
}

/// Parses bytes representing JSON formatted configuration of [`super::CargoLib`]
///
pub fn parse_config(raw_content: Vec<u8>) -> SingleErrVariantResult<CargoConfig, ParseConfigError> {
    let parsed: CargoConfig = serde_json::from_slice(&raw_content)
        .map_err(|e| SingleVariantError::Failure(ParseConfigError(e.to_string())))?;
    println!("Parsed config: {parsed:?}");
    Ok(parsed)
}

#[cfg(test)]
mod tests {

    use tracing::Level;

    use super::{CargoConfig, FoundationStorageApiConfig, LoggerConfig};

    #[test]
    fn test_parsing_debug_config() {
        let config_str = r#"{
            "log_level": "trace",
            "evs_runtime_mode": "DEBUG",
            "evs_url": "some_url",
            "sc_url": "some_url"
        }"#;

        let config: CargoConfig = serde_json::from_str(config_str).unwrap();

        assert_eq!(
            config,
            CargoConfig {
                fsa_config: FoundationStorageApiConfig {
                    evs_url: "some_url".to_owned(),
                    sc_url: "some_url".to_owned(),
                },
                logger_config: LoggerConfig {
                    log_level: Level::TRACE,
                    log_file: None
                }
            }
        )
    }

    #[test]
    fn test_parsing_prod_config() {
        let config_str = r#"{
            "log_level": "trace",
            "evs_runtime_mode": "PROD",
            "evs_url": "some_url",
            "sc_url": "some_url"
        }"#;

        let config: CargoConfig = serde_json::from_str(config_str).unwrap();

        assert_eq!(
            config,
            CargoConfig {
                fsa_config: FoundationStorageApiConfig {
                    evs_url: "some_url".to_owned(),
                    sc_url: "some_url".to_owned(),
                },
                logger_config: LoggerConfig {
                    log_level: Level::TRACE,
                    log_file: None
                }
            }
        )
    }
}
