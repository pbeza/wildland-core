//! This module consists of types and functions for creating configuration of [`super::CargoLib`].
//!
//! Configuration may be represented by a JSON like:
//! ```
//! # use wildland_cargo_lib::api::config::parse_config;
//! #
//! let config_json = r#"{
//!     "log_level": "trace",
//!     "log_use_ansi": false,
//!     "log_file_enabled": true,
//!     "log_file_path": "cargo_lib_log",
//!     "log_file_rotate_directory": ".",
//!     "evs_runtime_mode": "DEBUG",
//!     "evs_url": "some_url",
//!     "sc_url": "some_url"
//! }"#;
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
use tracing::{instrument, Level};

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
    fn get_log_use_ansi(&self) -> bool;
    fn get_log_file(&self) -> Option<String>;
    fn log_file_enabled(&self) -> bool;
    fn log_file_path(&self) -> Option<String>;
    fn log_file_rotate_directory(&self) -> Option<String>;
    fn oslog_category(&self) -> Option<String>;
    fn oslog_sybsystem(&self) -> Option<String>;

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

/// Structure representing configuration of [`super::CargoLib`].
/// It is used to create [`super::CargoLib`] instance.
/// It is created from JSON or from type implementing [`CargoCfgProvider`].\
/// See [`super::CargoLib`] for more details.
/// ```
/// # use wildland_cargo_lib::api::config::parse_config;
/// #
/// let config_json = r#"
/// {
///     "log_level": "debug",
///     "log_use_ansi": true
///     "log_file_path": "some_name",
///     "log_file_rotate_directory": Some("path"),
///     "log_file_enabled": True,
///     "oslog_sybsystem": None,
///     "oslog_category": None
///     "evs_url": "some_url",
///     "sc_url": "some_url"
/// }
/// "#;
///
#[derive(Debug, Deserialize, Clone, PartialEq, Eq, derivative::Derivative)]
#[derivative(Default(new = "true"))]
pub struct LoggerConfig {
    /// Minimum level of messages to get logged
    #[serde(deserialize_with = "log_level_deserialize")]
    #[derivative(Default(value = "Level::INFO"))]
    pub log_level: Level,

    /// If Enabled, the logger will use ansi sequences to style text
    /// not all platforms and receivers do support this feature. False by default.
    #[derivative(Default(value = "false"))]
    pub log_use_ansi: bool,

    /// Enables or disables file logging.
    #[derivative(Default(value = "false"))]
    pub log_file_enabled: bool,

    /// File describing where log entries should be mirrored to. This part
    /// defines the file path of the currently active log file.
    /// defaults to `cargolib_log`
    #[derivative(Default(value = "Some(\"cargolib_log\".to_string())"))]
    pub log_file_path: Option<String>,

    /// File describing where log entries should be mirrored to. This part
    /// defines the file directory where the rotation will happen.
    /// defaults to the current working directory.
    #[derivative(Default(value = "Some(\".\".to_string())"))]
    pub log_file_rotate_directory: Option<String>,

    /// Name of the system.log category. If Some() provided together with
    /// oslog_subsystem category, enables the oslog facility. If OsLog is enabled,
    /// then all other facilities are not initialized.
    #[derivative(Default(value = "None"))] // todo change to something else?
    pub oslog_category: Option<String>,

    /// Name of the system.log subsystem. If Some() provided together with
    /// oslog_category, enables the oslog facility. If OsLog is enabled,
    /// then all other facilities are not initialized.
    #[derivative(Default(value = "None"))]
    pub oslog_sybsystem: Option<String>,
}

impl LoggerConfig {
    /// Helper function used to determine platform capabilities
    /// Whenever the os log facilities are available and properly configured,
    /// returns `true`. However if the configuration does not contain all the data
    /// necessary to start the logger or the platform does not support logging
    /// to the OsLog (i.e. linux,windows) then `false` is returned.
    pub fn is_oslog_eligible(&self) -> bool {
        let correct_platform = cfg!(target_os = "macos") || cfg!(target_os = "ios");
        if self.oslog_category.is_some() && self.oslog_sybsystem.is_some() && correct_platform {
            return true;
        }
        false
    }

    /// Helper function used to determine platform capabilities
    /// Whenever the file log facilities are available and properly configured,
    /// returns `true`. However if the configuration does not contain all the
    /// fields necessary to start the logger or the platform does not support
    /// logging to the file (i.e. ios) then `false` is returned.
    #[instrument(skip(self))]
    pub fn is_file_eligible(&self) -> bool {
        if self.log_file_path.is_none() || self.log_file_rotate_directory.is_none() {
            return false;
        }
        if let Ok((_, _)) = self.filestrings_as_paths() {
            return true;
        }
        tracing::error!("file log can not be enabled with provided paths");
        false
    }

    /// Helper function to transform and check filepaths.
    /// Converts strings to std::path::PathBuf instances, and checks for their existance
    /// throws Error if the paths are not valid or do not exist. On success returns
    /// a tuple composed from "(filepath,rotatepath)" (taken from the config).
    #[instrument(skip(self))]
    pub fn filestrings_as_paths(&self) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
        // all unwraps should succeed because of the is_file_eligiible check
        // if it crashes on those, it means gods wanted it this way...
        let file_path = std::path::PathBuf::from(self.log_file_path.clone().unwrap());
        let dir_path = std::path::PathBuf::from(self.log_file_rotate_directory.clone().unwrap());

        if !file_path.exists() {
            anyhow::bail!("provided path: {} does not exist!", file_path.display());
        }

        if !dir_path.exists() {
            anyhow::bail!("provided path: {} does not exist!", dir_path.display());
        }

        Ok((file_path, dir_path))
    }

    pub fn validate_config_level(&mut self) {
        let detect_if_debug_build = cfg!(debug_assertions);
        if self.log_level > Level::INFO && !detect_if_debug_build {
            self.log_level = Level::INFO;
            tracing::warn!("log level set to INFO because of release build");
        }
    }
}

/// Helper function for handling deserializing `log_level` field
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
            log_use_ansi: config_provider.get_log_use_ansi(),
            log_file_path: config_provider.get_log_file(),
            log_file_enabled: config_provider.log_file_enabled(),
            log_file_rotate_directory: config_provider.log_file_rotate_directory(),
            oslog_category: config_provider.oslog_category(),
            oslog_sybsystem: config_provider.oslog_sybsystem(),
        },
        fsa_config: FoundationStorageApiConfig {
            evs_url: config_provider.get_evs_url(),
            sc_url: config_provider.get_sc_url(),
        },
    })
}

/// Parses bytes representing JSON formatted configuration of [`super::CargoLib`]
/// into an instance of [`CargoConfig`]
/// The `settings` must be a string with JSON formatted configuration.
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
            "log_use_ansi": false,
            "log_file_enabled": true,
            "log_file_path": "cargo_lib_log",
            "log_file_rotate_directory": ".",
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
                    log_use_ansi: false,
                    log_file_path: Some("cargo_lib_log".to_owned()),
                    log_file_rotate_directory: Some(".".to_owned()),
                    log_file_enabled: true,
                    oslog_category: None,
                    oslog_sybsystem: None,
                }
            }
        )
    }

    #[test]
    fn test_parsing_prod_config() {
        let config_str = r#"{
            "log_level": "trace",
            "log_use_ansi": true,
            "log_file_enabled": false,
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
                    log_use_ansi: true,
                    log_file_path: None,
                    log_file_rotate_directory: None,
                    log_file_enabled: false,
                    oslog_category: None,
                    oslog_sybsystem: None,
                }
            }
        )
    }
}
