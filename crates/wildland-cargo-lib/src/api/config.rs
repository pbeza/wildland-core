use serde::{Deserialize, Serialize};

pub trait CargoCfgProvider {
    fn get_config(&self) -> Vec<u8>;
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl LogLevel {
    fn info() -> Self {
        Self::Info
    }
}

impl From<LogLevel> for tracing::Level {
    fn from(l: LogLevel) -> Self {
        match l {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct LoggerCfg {
    #[serde(default = "LogLevel::info")]
    pub log_level: LogLevel,
    #[serde(default = "default_log_file")]
    pub log_file: String,
}

fn default_log_file() -> String {
    "cargo.log".to_owned()
}

fn default_logger() -> LoggerCfg {
    LoggerCfg {
        log_level: LogLevel::info(),
        log_file: default_log_file(),
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub(crate) struct CargoConfig {
    #[serde(default = "default_logger")]
    pub logger: LoggerCfg,
}

#[cfg(test)]
mod tests {
    use crate::api::config::LoggerCfg;

    use super::{CargoConfig, LogLevel};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_deserialize_default() {
        let json = r#"
            {}
        "#;
        let config: CargoConfig = serde_json::from_str(json).unwrap();
        let expected = CargoConfig {
            logger: LoggerCfg {
                log_level: LogLevel::Info,
                log_file: "cargo.log".to_owned(),
            },
        };
        assert_eq!(expected, config);
    }

    #[test]
    fn test_deserialize_empty_cfg() {
        let json = r#"
            {
                "logger": {
                    "log_level": "trace",
                    "log_file": "some_corex.log"
                }
            }
        "#;
        let config: CargoConfig = serde_json::from_str(json).unwrap();
        let expected = CargoConfig {
            logger: LoggerCfg {
                log_level: LogLevel::Trace,
                log_file: "some_corex.log".to_owned(),
            },
        };
        assert_eq!(expected, config);
    }

    #[test]
    fn test_deserialize_partial_cfg() {
        let json = r#"
            {
                "logger": {
                    "log_level": "trace"
                }
            }
        "#;
        let config: CargoConfig = serde_json::from_str(json).unwrap();
        let expected = CargoConfig {
            logger: LoggerCfg {
                log_level: LogLevel::Trace,
                log_file: "cargo.log".to_owned(),
            },
        };
        assert_eq!(expected, config);
    }
}
