use crate::api::config::LoggerConfig;
use std::io::{self};
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};

pub(crate) fn init_subscriber(cfg: LoggerConfig) -> anyhow::Result<()> {
    let mut cfg = cfg;

    // check if we are using release or debug build and adjust the level
    cfg.validate_config_level();

    // check which logging type should be used, construct the subscriber and
    // init it
    if cfg.is_oslog_eligible() {
        nondefault_oslog(&cfg)?;
    } else if cfg.is_file_eligible() {
        default_with_file_copy(&cfg)?;
    } else {
        default_without_file_copy(&cfg)?;
    }
    tracing::debug!("logger initialized");
    Ok(())
}

pub fn default_with_file_copy(cfg: &LoggerConfig) -> anyhow::Result<()> {
    let Ok((filepath,rotatedir)) = cfg.filestrings_as_paths() else {
        // we do not have subscriber yet, so lets just print to stderr
        std::eprintln!("requested logger with file but paths provided are not valid or does not exist");
        anyhow::bail!("requested logger with file but paths provided are not valid or does not exist");
    };

    let file_appender = tracing_appender::rolling::hourly(filepath, rotatedir);
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(cfg.log_level.into()))
        .with(
            fmt::Layer::new()
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(cfg.log_use_ansi)
                .with_writer(file_appender),
        )
        .with(
            fmt::Layer::new()
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(cfg.log_use_ansi)
                .with_writer(std::io::stderr),
        );
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global multilogger instance"); // unrecoverable
    tracing::debug!("initialized multilogger");
    Ok(())
}

pub fn default_without_file_copy(cfg: &LoggerConfig) -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(cfg.log_level.into()))
        .with(
            fmt::Layer::new()
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(cfg.log_use_ansi)
                .with_writer(io::stdout),
        );
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global stderr-log instance"); // unrecoverable
    tracing::debug!("initialized stderr logger");
    Ok(())
}

pub fn nondefault_oslog(cfg: &LoggerConfig) -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_oslog::OsLogger::new(
            cfg.oslog_sybsystem.clone().unwrap(),
            cfg.oslog_sybsystem.clone().unwrap(),
        ));
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global oslog instance"); // unrecoverable
    tracing::debug!("initialized oslog logger");
    Ok(())
}
