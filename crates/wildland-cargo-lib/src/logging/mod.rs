static mut GUARDS: Vec<WorkerGuard> = vec![];

use std::str::FromStr;

use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_core::LevelFilter;
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

pub fn init_subscriber(log_level: String, log_file: Option<String>) -> Result<(), String> {
    let fmt_layer = fmt::layer().with_target(false).with_level(true);
    let level_filter =
        LevelFilter::from_level(Level::from_str(log_level.as_str()).map_err(|e| e.to_string())?);

    let logger = tracing_subscriber::registry()
        .with(level_filter)
        .with(fmt_layer);
    if let Some(log_file) = log_file {
        let file_appender = tracing_appender::rolling::hourly(".", log_file);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // Currently required;
        // TODO:WILX-213: i will figure it out later.
        unsafe {
            GUARDS.push(guard);
        }
        logger
            .with(Some(fmt::Layer::new().with_writer(non_blocking)))
            .init();
    } else {
        logger.init();
    }
    Ok(())
}
