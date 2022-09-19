use serde::{Deserialize, Serialize};

pub trait CargoCfgProvider {
    fn get_config(&self) -> Vec<u8>;
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub(crate) struct CargoConfig {
    #[serde(default = "LogLevel::info")]
    log_level: LogLevel,
}

#[cfg(test)]
mod tests {
    use super::{CargoConfig, LogLevel};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_deserialize_default() {
        let json = r#"
            {}
        "#;
        let config: CargoConfig = serde_json::from_str(json).unwrap();
        let expected = CargoConfig {
            log_level: LogLevel::Info,
        };
        assert_eq!(expected, config);
    }

    #[test]
    fn test_deserialize_non_default() {
        let json = r#"
            {
                "log_level": "trace"
            }
        "#;
        let config: CargoConfig = serde_json::from_str(json).unwrap();
        let expected = CargoConfig {
            log_level: LogLevel::Trace,
        };
        assert_eq!(expected, config);
    }
}
