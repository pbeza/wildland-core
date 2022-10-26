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

use std::io::{self};
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};

use crate::api::config::LoggerConfig;

pub(crate) fn init_subscriber(cfg: LoggerConfig) -> Result<(), String> {
    let is_ansi = std::env::vars_os().any(|(k, _)| k == "ANSI");
    if let Some(logfile) = cfg.log_file {
        default_with_file_copy(cfg.log_level, logfile, is_ansi);
    } else {
        default_without_file_copy(cfg.log_level, is_ansi);
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
