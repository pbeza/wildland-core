use serde::Deserialize;
use thiserror::Error;

use crate::errors::{SingleErrVariantResult, SingleVariantError};

pub trait CargoCfgProvider {
    fn get_log_level(&self) -> String;
    fn get_log_file(&self) -> Option<String>;
    fn get_evs_url(&self) -> String;
}

#[derive(PartialEq, Eq, Error, Debug, Clone)]
#[error("Config parse error: {0}")]
pub struct ParseConfigError(pub String);

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct FoundationStorageApiConfig {
    pub evs_url: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct LoggerConfig {
    pub log_level: String,
    pub log_file: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct CargoConfig {
    #[serde(flatten)]
    pub fsa_config: FoundationStorageApiConfig,
    #[serde(flatten)]
    pub logger_config: LoggerConfig,
}

pub fn collect_config(config_provider: &'static dyn CargoCfgProvider) -> CargoConfig {
    CargoConfig {
        logger_config: LoggerConfig {
            log_level: config_provider.get_log_level(),
            log_file: config_provider.get_log_file(),
        },
        fsa_config: FoundationStorageApiConfig {
            evs_url: config_provider.get_evs_url(),
        },
    }
}

pub fn parse_config(raw_content: Vec<u8>) -> SingleErrVariantResult<CargoConfig, ParseConfigError> {
    let parsed: CargoConfig = serde_json::from_slice(&raw_content)
        .map_err(|e| SingleVariantError::Failure(ParseConfigError(e.to_string())))?;
    println!("Parsed config: {parsed:?}");
    Ok(parsed)
}

#[cfg(test)]
mod tests {

    use super::{CargoConfig, FoundationStorageApiConfig, LoggerConfig};

    #[test]
    fn test_parsing_debug_config() {
        let config_str = r#"{
            "log_level": "trace",
            "evs_runtime_mode": "DEBUG",
            "evs_url": "some_url",
            "evs_credentials_payload": "some_payload"
        }"#;

        let config: CargoConfig = serde_json::from_str(config_str).unwrap();

        assert_eq!(
            config,
            CargoConfig {
                fsa_config: FoundationStorageApiConfig {
                    evs_url: "some_url".to_owned(),
                },
                logger_config: LoggerConfig {
                    log_level: "trace".to_owned(),
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
            "evs_credentials_payload": "some_payload"
        }"#;

        let config: CargoConfig = serde_json::from_str(config_str).unwrap();

        assert_eq!(
            config,
            CargoConfig {
                fsa_config: FoundationStorageApiConfig {
                    evs_url: "some_url".to_owned(),
                },
                logger_config: LoggerConfig {
                    log_level: "trace".to_owned(),
                    log_file: None
                }
            }
        )
    }
}
