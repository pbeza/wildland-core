static mut GUARDS: Vec<WorkerGuard> = vec![];

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub fn init_subscriber() -> anyhow::Result<()> {
    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    let filename = std::env::var("RUST_LOGFILE").unwrap_or_else(|_| String::from("corex.log"));

    let file_appender = tracing_appender::rolling::hourly(".", filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Currently required;
    // TODO:WILX-213: i will figure it out later.
    unsafe {
        GUARDS.push(guard);
    }

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(Some(fmt::Layer::new().with_writer(non_blocking)))
        .init();
    Ok(())
}
