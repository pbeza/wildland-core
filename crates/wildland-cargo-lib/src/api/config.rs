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
//! let config_json = r#"{
//!     "log_level": "trace",
//!     "log_use_ansi": false,
//!     "log_file_enabled": true,
//!     "log_file_path": "cargo_lib_log",
//!     "log_file_rotate_directory": ".",
//!     "evs_url": "some_url",
//!     "sc_url": "some_url"
//! }"#;
//!
//! let _  = parse_config(config_json.as_bytes().to_vec()).unwrap();
//! ```
//!
//! It can be also provided via type implementing [`CargoCfgProvider`].

use std::path::PathBuf;
use std::str::FromStr;

use derivative::Derivative;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use thiserror::Error;
use tracing::Level;

const DEV_DEFAULT_EVS_URL: &str = "https://evs.cargo.wildland.dev/";
const DEV_DEFAULT_SC_URL: &str = "https://storage-controller.cargo.wildland.dev/";

#[repr(C)]
pub enum FoundationCloudMode {
    Dev,
}

impl From<FoundationCloudMode> for FoundationStorageApiConfig {
    fn from(mode: FoundationCloudMode) -> Self {
        match mode {
            FoundationCloudMode::Dev => FoundationStorageApiConfig {
                evs_url: DEV_DEFAULT_EVS_URL.to_string(),
                sc_url: DEV_DEFAULT_SC_URL.to_string(),
            },
        }
    }
}

pub trait CargoCfgProvider {
    fn get_use_logger(&self) -> bool;
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
    fn get_log_file_enabled(&self) -> bool;
    fn get_log_file_path(&self) -> Option<String>;
    fn get_log_file_rotate_directory(&self) -> Option<String>;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn get_oslog_category(&self) -> Option<String>;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn get_oslog_subsystem(&self) -> Option<String>;

    fn get_foundation_cloud_env_mode(&self) -> FoundationCloudMode;
}

#[derive(PartialEq, Eq, Error, Debug, Clone)]
#[repr(C)]
pub enum ParseConfigError {
    #[error("Config parse error: {0}")]
    Error(String),
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Derivative)]
#[derivative(Default(new = "true"))]
pub struct FoundationStorageApiConfig {
    #[serde(default = "default_evs_url")]
    #[derivative(Default(value = "default_evs_url()"))]
    pub evs_url: String,
    #[serde(default = "default_sc_url")]
    #[derivative(Default(value = "default_sc_url()"))]
    pub sc_url: String,
}

fn default_evs_url() -> String {
    // TODO for now default is DEV
    // in the future we might distinguish debug and release builds in order to choose environment
    DEV_DEFAULT_EVS_URL.to_owned()
}

fn default_sc_url() -> String {
    // TODO for now default is DEV
    // in the future we might distinguish debug and release builds in order to choose environment
    DEV_DEFAULT_SC_URL.to_owned()
}

fn bool_default_as_true() -> bool {
    true
}

fn serde_logger_default_path() -> PathBuf {
    PathBuf::from("cargo_lib_log")
}

fn serde_logger_default_rot_dir() -> PathBuf {
    PathBuf::from(".")
}

/// Structure representing configuration of [`super::CargoLib`].
/// It is used to create [`super::CargoLib`] instance.
/// It is created from JSON or from type implementing [`CargoCfgProvider`].
/// This structure provides default values for all fields and can be constructed
/// by either LoggerConfig::new() or LoggerConfig::default(). Those two calls
/// are equivalent to each other.
/// ```
/// # use wildland_cargo_lib::api::config::parse_config;
/// #
/// let config_json = r#"
/// {
///     "log_level": "debug",
///     "log_use_ansi": true,
///     "log_file_path": "some_name",
///     "log_file_rotate_directory": ".",
///     "log_file_enabled": true,
///     "evs_url": "some_url",
///     "sc_url": "some_url"
/// }
/// "#;
/// let parsed_cfg = parse_config(config_json.as_bytes().to_vec()).unwrap();
///
///
#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Derivative)]
#[derivative(Default(new = "true"))]
pub struct LoggerConfig {
    /// switch to disable the logger facility.
    /// If set to false, the logger will be disabled.
    /// usefull for cases where the client wants to use its own tracing
    /// subscriber object or want to enable it from the outside.
    /// Default: true
    ///
    /// Most users will want to leave it defaulted to true, especially users
    /// of the bindings, as they will not be able to create subscriber
    /// externally.
    ///
    /// In case its false, all the log configs are not used nor the subscriber
    /// is created.
    #[serde(default = "bool_default_as_true")]
    #[derivative(Default(value = "true"))]
    pub use_logger: bool,

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
    #[serde(default = "serde_logger_default_path")]
    #[derivative(Default(value = "serde_logger_default_path()"))]
    pub log_file_path: PathBuf,

    /// File describing where log entries should be mirrored to. This part
    /// defines the file directory where the rotation will happen.
    /// defaults to the current working directory.
    #[serde(default = "serde_logger_default_rot_dir")]
    #[derivative(Default(value = "serde_logger_default_rot_dir()"))]
    pub log_file_rotate_directory: PathBuf,

    /// Name of the system.log category. If Some() provided together with
    /// oslog_subsystem category, enables the oslog facility. If OsLog is enabled,
    /// then all other facilities are not initialized.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[derivative(Default(value = "None"))]
    pub oslog_category: Option<String>,

    /// Name of the system.log subsystem. If Some() provided together with
    /// oslog_category, enables the oslog facility. If OsLog is enabled,
    /// then all other facilities are not initialized.

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[derivative(Default(value = "None"))]
    pub oslog_subsystem: Option<String>,
}

impl LoggerConfig {
    /// Helper function used to determine platform capabilities
    /// Whenever the os log facilities are available and properly configured,
    /// returns `true`. However if the configuration does not contain all the data
    /// necessary to start the logger or the platform does not support logging
    /// to the OsLog (i.e. linux,windows) then `false` is returned.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn is_oslog_eligible(&self) -> bool {
        if self.oslog_category.is_some() && self.oslog_subsystem.is_some() {
            return true;
        }
        false
    }

    /// Helper function used to determine platform capabilities
    /// Whenever the file log facilities are available and properly configured,
    /// returns `true`. However if the configuration uses paths that do not exist
    /// we will fail to initialize the logger and return `false`.
    pub fn is_file_eligible(&self) -> bool {
        if !self.log_file_enabled {
            return false;
        }

        let filepath = self.log_file_path.clone();
        let dirpath = self.log_file_rotate_directory.clone();

        // if filepaths are not existing, whetever provided or defaults, we are
        // not eligible to start file logging subscriber
        if !filepath.exists() || !dirpath.exists() {
            std::eprintln!("file log paths do not exist, we failed to create logging subscriber");
            return false;
        }

        true
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

impl CargoConfig {
    pub fn override_evs_url(&mut self, new_evs_url: String) {
        self.fsa_config.evs_url = new_evs_url;
    }

    pub fn override_sc_url(&mut self, new_sc_url: String) {
        self.fsa_config.sc_url = new_sc_url;
    }
}

/// Uses an implementation of [`CargoCfgProvider`] to collect a configuration storing structure ([`CargoConfig`])
/// which then can be passed to [`super::cargo_lib::create_cargo_lib`] in order to instantiate main API object ([`super::CargoLib`])
///
#[tracing::instrument(level = "debug", skip_all)]
pub fn collect_config(
    config_provider: &'static dyn CargoCfgProvider,
) -> Result<CargoConfig, ParseConfigError> {
    Ok(CargoConfig {
        logger_config: LoggerConfig {
            use_logger: config_provider.get_use_logger(),
            log_level: Level::from_str(config_provider.get_log_level().as_str())
                .map_err(|e| ParseConfigError::Error(e.to_string()))?,
            log_use_ansi: config_provider.get_log_use_ansi(),
            log_file_path: PathBuf::from(
                config_provider
                    .get_log_file_path()
                    .unwrap_or_else(|| serde_logger_default_path().to_string_lossy().to_string()),
            ),
            log_file_enabled: config_provider.get_log_file_enabled(),
            log_file_rotate_directory: PathBuf::from(
                config_provider
                    .get_log_file_rotate_directory()
                    .unwrap_or_else(|| {
                        serde_logger_default_rot_dir().to_string_lossy().to_string()
                    }),
            ),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            oslog_category: config_provider.get_oslog_category(),

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            oslog_subsystem: config_provider.get_oslog_subsystem(),
        },
        fsa_config: config_provider.get_foundation_cloud_env_mode().into(),
    })
}

/// Parses bytes representing JSON formatted configuration of [`super::CargoLib`]
/// into an instance of [`CargoConfig`]
/// The `settings` must be a string with JSON formatted configuration.
///
#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_config(raw_content: Vec<u8>) -> Result<CargoConfig, ParseConfigError> {
    let parsed: CargoConfig =
        serde_json::from_slice(&raw_content).map_err(|e| ParseConfigError::Error(e.to_string()))?;
    println!("Parsed config: {parsed:?}");
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
                    use_logger: true,
                    log_level: Level::TRACE,
                    log_use_ansi: false,
                    log_file_path: PathBuf::from("cargo_lib_log"),
                    log_file_rotate_directory: PathBuf::from("."),
                    log_file_enabled: true,
                }
            }
        )
    }

    #[test]
    fn test_parsing_prod_config() {
        let config_str = r#"{
            "log_level": "trace",
            "log_use_ansi": true,
            "log_file_enabled": false
        }"#;

        let config: CargoConfig = serde_json::from_str(config_str).unwrap();

        assert_eq!(
            config,
            CargoConfig {
                fsa_config: FoundationStorageApiConfig {
                    evs_url: "https://evs.cargo.wildland.dev/".to_owned(),
                    sc_url: "https://storage-controller.cargo.wildland.dev/".to_owned(),
                },
                logger_config: LoggerConfig {
                    use_logger: true,
                    log_level: Level::TRACE,
                    log_use_ansi: true,
                    log_file_path: LoggerConfig::default().log_file_path,
                    log_file_rotate_directory: LoggerConfig::default().log_file_rotate_directory,
                    log_file_enabled: false,
                }
            }
        )
    }
}
