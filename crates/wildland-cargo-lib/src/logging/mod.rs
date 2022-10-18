use std::io::{self};
use tracing::Level;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};

pub(crate) fn init_subscriber(log_level: Level, log_file: Option<String>) -> Result<(), String> {
    let is_ansi = std::env::vars_os().any(|(k, _)| k == "ANSI");
    if let Some(logfile) = log_file {
        default_with_file_copy(log_level, logfile, is_ansi);
    } else {
        default_without_file_copy(log_level, is_ansi);
    };
    tracing::debug!("logger initialized");
    Ok(())
}

pub fn default_with_file_copy(log_level: tracing::Level, filename: String, is_ansi: bool) {
    let file_appender = tracing_appender::rolling::hourly(".", filename);
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(
            fmt::Layer::new()
                .with_ansi(false)
                .with_writer(file_appender),
        )
        .with(
            fmt::Layer::new()
                .with_ansi(is_ansi)
                .with_writer(std::io::stderr),
        );
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global multilogger instance");
    tracing::debug!("initialized multilogger");
}

pub fn default_without_file_copy(log_level: tracing::Level, is_ansi: bool) {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(fmt::Layer::new().with_ansi(is_ansi).with_writer(io::stdout));
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global stderr-log instance");
    tracing::debug!("initialized stderr logger");
}
