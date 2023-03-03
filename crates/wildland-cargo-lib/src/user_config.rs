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

/// This module provides an interface to the persistent storage of the
/// user-specific data that should be available across all the user's
/// devices.
///


use std::collections::HashMap;
use redis::{Commands, ConnectionLike};

pub(crate) use crate::errors::config::UserConfigDatabaseError;

pub(crate) type DbClient = r2d2::Pool<redis::Client>;

fn redis_to_cargolib_err(err: redis::RedisError) -> UserConfigDatabaseError {
    UserConfigDatabaseError::Generic(format!("Redis error: {err}"))
}

fn r2d2_to_cargolib_err(err: r2d2::Error) -> UserConfigDatabaseError {
    UserConfigDatabaseError::Generic(format!("Catlib connection pool error: {err}"))
}

fn handle_key_prefix(db: &RedisDb, key: &str) -> String {
    if !db.key_prefix.is_empty() {
        format!("{}:{}", db.key_prefix, key)
    } else {
        key.to_owned()
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn ping(db: &RedisDb) -> Result<bool, UserConfigDatabaseError> {
    Ok(db
        .client
        .get()
        .map_err(r2d2_to_cargolib_err)?
        .check_connection())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn find_keys(db: &RedisDb, query: &str) -> Result<Vec<String>, UserConfigDatabaseError> {
    // TODO [COR-72]: use scan, not keys (optimisation)
    db.client
        .get()
        .map_err(r2d2_to_cargolib_err)?
        .keys(handle_key_prefix(db, query))
        .map_err(redis_to_cargolib_err)
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn query_get(
    db: &RedisDb,
    query: &str,
) -> Result<HashMap<String, Option<String>>, UserConfigDatabaseError> {
    let keys: Vec<String> = find_keys(db, query)?;

    if keys.is_empty() {
        return Err(UserConfigDatabaseError::NoRecordsFound(query.to_owned()));
    };

    let values: Vec<Option<String>> = db
        .client
        .get()
        .map_err(r2d2_to_cargolib_err)?
        .get(keys.clone())
        .map_err(redis_to_cargolib_err)?;

    Ok(keys.into_iter().zip(values).collect())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn get(db: &RedisDb, key: &str) -> Result<Option<String>, UserConfigDatabaseError> {
    let key = handle_key_prefix(db, key);

    let value = db
        .client
        .get()
        .map_err(r2d2_to_cargolib_err)?
        .get(key)
        .map_err(redis_to_cargolib_err)?;

    Ok(value)
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn set(db: &RedisDb, key: &str, data: String) -> Result<(), UserConfigDatabaseError> {
    db.client
        .get()
        .map_err(r2d2_to_cargolib_err)?
        .set(handle_key_prefix(db, key), data)
        .map_err(redis_to_cargolib_err)?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn delete(db: &RedisDb, key: &str) -> Result<(), UserConfigDatabaseError> {
    db.client
        .get()
        .map_err(r2d2_to_cargolib_err)?
        .del(handle_key_prefix(db, key))
        .map_err(redis_to_cargolib_err)?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn db_conn(connection_string: String) -> Result<DbClient, UserConfigDatabaseError> {
    let client = redis::Client::open(connection_string).map_err(redis_to_cargolib_err)?;

    // Do not throw exception if there's no connection to the backend during
    // initialisation. The pool size will increase during an attempt to
    // interact with the backend.
    let min_connection_pool_size = Some(0);

    r2d2::Pool::builder()
        .min_idle(min_connection_pool_size)
        .idle_timeout(Some(std::time::Duration::from_secs(5 * 60)))
        .connection_timeout(std::time::Duration::from_secs(10))
        .build(client)
        .map_err(r2d2_to_cargolib_err)
}

pub(crate) fn is_alive(db: &RedisDb) -> Result<bool, UserConfigDatabaseError> {
    ping(db)
}

#[derive(Clone)]
pub(crate) struct RedisDb {
    pub client: DbClient,
    pub key_prefix: String,
}

#[derive(Clone)]
pub struct UserMultideviceConfig {
    db: RedisDb
}

impl UserMultideviceConfig {
    pub fn new(redis_url: String, key_prefix: String) -> Result<Self, UserConfigDatabaseError> {
        let client = db_conn(redis_url)?;
        let db = RedisDb {
            client,
            key_prefix
        };
        Ok(Self {
            db
        })
    }
}

impl Default for UserMultideviceConfig {
    fn default() -> Self {
        use std::env;
        let redis_url =
            env::var("CARGO_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".into());
        let db_prefix = env::var("CARGO_DB_KEY_PREFIX").unwrap_or_else(|_| "".into());

        UserMultideviceConfig::new(redis_url, db_prefix).expect("Default database endpoint is not available")
    }
}

#[cfg(test)]
mod tests {

}
