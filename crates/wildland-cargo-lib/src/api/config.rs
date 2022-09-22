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

#[derive(Debug, Deserialize, Clone)]
pub struct CargoConfig {
    log_level: String,
    log_file: Option<String>,
}

impl CargoCfgProvider for CargoConfig {
    fn get_log_level(&self) -> String {
        self.log_level.clone()
    }

    fn get_log_file(&self) -> Option<String> {
        self.log_file.clone()
    }
}

pub fn collect_config(config_provider: &'static dyn CargoCfgProvider) -> CargoConfig {
    CargoConfig {
        log_level: config_provider.get_log_level(),
        log_file: config_provider.get_log_file(),
    }
}

pub fn parse_config(raw_content: Vec<u8>) -> SingleErrVariantResult<CargoConfig, ParseConfigError> {
    let parsed: CargoConfig = serde_json::from_slice(&raw_content)
        .map_err(|e| SingleVariantError::Failure(ParseConfigError(e.to_string())))?;
    println!("Parsed config: {parsed:?}");
    Ok(parsed)
}