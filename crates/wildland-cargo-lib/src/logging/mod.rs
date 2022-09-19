static mut GUARDS: Vec<WorkerGuard> = vec![];

use tracing_appender::non_blocking::WorkerGuard;
use tracing_core::LevelFilter;
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

use crate::api::config::LoggerCfg;

pub fn init_subscriber(logger_cfg: LoggerCfg) -> anyhow::Result<()> {
    let fmt_layer = fmt::layer().with_target(false).with_level(true);
    let level_filter = LevelFilter::from_level(logger_cfg.log_level.into());
    let file_appender = tracing_appender::rolling::hourly(".", logger_cfg.log_file);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Currently required;
    // TODO:WILX-213: i will figure it out later.
    unsafe {
        GUARDS.push(guard);
    }

    tracing_subscriber::registry()
        .with(level_filter)
        .with(fmt_layer)
        .with(Some(fmt::Layer::new().with_writer(non_blocking)))
        .init();
    Ok(())
}
