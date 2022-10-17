use std::io::{self};
use tracing::Level;
use tracing_subscriber::{
    fmt, fmt::writer::MakeWriterExt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter,
};

pub(crate) fn init_subscriber(log_level: Level, log_file: Option<String>) -> Result<(), String> {
    if let Some(logfile) = log_file {
        default_with_file_copy(log_level, logfile);
    } else {
        default_without_file_copy(log_level);
    };

    tracing::debug!("logger initialized");
    Ok(())
}

pub fn default_with_file_copy(log_level: tracing::Level, filename: String) {
    let file_appender = tracing_appender::rolling::hourly(".", filename);
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(fmt::Layer::new().with_writer(file_appender.and(std::io::stderr)));
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global multilogger instance");
    tracing::debug!("initialized multilogger");
}

pub fn default_without_file_copy(log_level: tracing::Level) {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(fmt::Layer::new().with_writer(io::stdout));
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global stderr-log instance");
    tracing::debug!("initialized stderr logger");
}
