//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::io;

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter};

use crate::api::config::LoggerConfig;

pub(crate) fn init_subscriber(cfg: LoggerConfig) -> anyhow::Result<()> {
    if !cfg.use_logger {
        eprintln!("default log subscriber disabled by config!");
        return Ok(());
    }

    let mut cfg = cfg;

    // check if we are using release or debug build and adjust the level
    cfg.validate_config_level();

    // check which logging type should be used, construct the subscriber and
    // init it

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    if cfg.is_oslog_eligible() {
        nondefault_oslog(&cfg)?;
        tracing::info!("logger initialized");
        return Ok(());
    }

    if cfg.is_file_eligible() {
        default_with_file_copy(&cfg)?;
    } else {
        default_without_file_copy(&cfg)?;
    }
    tracing::info!("logger initialized");
    Ok(())
}

pub fn default_with_file_copy(cfg: &LoggerConfig) -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::hourly(
        cfg.log_file_path.clone(),
        cfg.log_file_rotate_directory.clone(),
    );
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(cfg.log_level.into()))
        .with(
            fmt::Layer::new()
                .with_file(true)
                .with_line_number(true)
                .with_ansi(cfg.log_use_ansi)
                .with_writer(file_appender),
        )
        .with(
            fmt::Layer::new()
                .with_file(true)
                .with_line_number(true)
                .with_ansi(cfg.log_use_ansi)
                .with_writer(std::io::stderr),
        );
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global multilogger instance"); // unrecoverable
    tracing::info!("initialized multilogger");
    Ok(())
}

pub fn default_without_file_copy(cfg: &LoggerConfig) -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(cfg.log_level.into()))
        .with(
            fmt::Layer::new()
                .with_file(true)
                .with_line_number(true)
                .with_ansi(cfg.log_use_ansi)
                .with_writer(io::stdout),
        );
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global stderr-log instance"); // unrecoverable
    tracing::info!("initialized stderr logger");
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn nondefault_oslog(cfg: &LoggerConfig) -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_oslog::OsLogger::new(
            cfg.oslog_subsystem.clone().unwrap(),
            cfg.oslog_category.clone().unwrap(),
        ));
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set a global oslog instance"); // unrecoverable
    tracing::info!("initialized oslog logger");
    Ok(())
}
