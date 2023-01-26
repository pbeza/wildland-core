use std::collections::HashMap;

use redis::Commands;
use wildland_corex::catlib_service::error::{CatlibError, CatlibResult};

use crate::error::to_catlib_error;
use crate::RedisDb;

fn handle_key_prefix(db: RedisDb, mut key: String) -> String {
    if !db.key_prefix.is_empty() {
        key = format!("{}:{}", db.key_prefix, key)
    }

    key
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn find_keys(db: RedisDb, query: String) -> CatlibResult<Vec<String>> {
    // todotodo: use scan, not keys (optimisation)
    db.client
        .borrow_mut()
        .keys(handle_key_prefix(db.clone(), query).as_str())
        .map_err(to_catlib_error)
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn query_get(
    db: RedisDb,
    query: String,
) -> CatlibResult<HashMap<String, Option<String>>> {
    let keys: Vec<String> = find_keys(db.clone(), query)?;

    if keys.is_empty() {
        return Err(CatlibError::NoRecordsFound);
    };

    let values: Vec<Option<String>> = db
        .client
        .borrow_mut()
        .get(keys.clone())
        .map_err(to_catlib_error)?;

    Ok(keys.iter().cloned().zip(values).collect())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn get(db: RedisDb, key: String) -> CatlibResult<(String, Option<String>)> {
    let key = handle_key_prefix(db.clone(), key);

    let value = db
        .client
        .borrow_mut()
        .get(key.clone())
        .map_err(to_catlib_error)?;

    Ok((key, value))
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn set(db: RedisDb, key: String, data: String) -> CatlibResult<()> {
    db.client
        .borrow_mut()
        .set(handle_key_prefix(db.clone(), key), data)
        .map_err(to_catlib_error)?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn delete(db: RedisDb, key: String) -> CatlibResult<()> {
    db.client
        .borrow_mut()
        .del(handle_key_prefix(db.clone(), key))
        .map_err(to_catlib_error)?;

    Ok(())
}
