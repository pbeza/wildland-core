use std::collections::HashMap;

use redis::Commands;
use wildland_corex::catlib_service::error::{CatlibError, CatlibResult};

use crate::RedisDb;

fn handle_key_prefix(db: &RedisDb, mut key: String) -> String {
    if !db.key_prefix.is_empty() {
        key = format!("{}:{}", db.key_prefix, key)
    }

    key
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn find_keys(db: &RedisDb, query: String) -> CatlibResult<Vec<String>> {
    // TODO [COR-72]: use scan, not keys (optimisation)
    db.client
        .get()?
        .keys(handle_key_prefix(db, query))
        .map_err(|e| e.into())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn query_get(
    db: &RedisDb,
    query: String,
) -> CatlibResult<HashMap<String, Option<String>>> {
    let keys: Vec<String> = find_keys(db, query)?;

    if keys.is_empty() {
        return Err(CatlibError::NoRecordsFound);
    };

    let values: Vec<Option<String>> = db.client.get()?.get(keys.clone())?;

    Ok(keys.into_iter().zip(values).collect())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn get(db: &RedisDb, key: String) -> CatlibResult<Option<String>> {
    let key = handle_key_prefix(db, key);

    let value = db.client.get()?.get(key)?;

    Ok(value)
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn set(db: &RedisDb, key: String, data: String) -> CatlibResult<()> {
    db.client.get()?.set(handle_key_prefix(db, key), data)?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn delete(db: &RedisDb, key: String) -> CatlibResult<()> {
    db.client.get()?.del(handle_key_prefix(db, key))?;

    Ok(())
}
