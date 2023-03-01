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

pub(crate) mod commands;

use std::path::PathBuf;
use std::str::FromStr;

use commands::*;
use wildland_corex::catlib_service::entities::{
    BridgeManifest,
    ContainerManifest,
    ForestManifest,
    StorageManifest,
};
use wildland_corex::ContainerPaths;

use super::*;
use crate::bridge::BridgeData;
use crate::container::ContainerData;
use crate::forest::ForestData;
use crate::storage::StorageData;

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn db_conn(connection_string: String) -> CatlibResult<DbClient> {
    let client = redis::Client::open(connection_string).map_err(redis_to_catlib_err)?;

    // Do not throw exception if there's no connection to the backend during
    // initialisation. The pool size will increase during an attempt to
    // interact with the backend.
    let min_connection_pool_size = Some(0);

    r2d2::Pool::builder()
        .min_idle(min_connection_pool_size)
        .idle_timeout(Some(std::time::Duration::from_secs(5 * 60)))
        .connection_timeout(std::time::Duration::from_secs(10))
        .build(client)
        .map_err(r2d2_to_catlib_err)
}

pub(crate) fn is_alive(db: &RedisDb) -> CatlibResult<bool> {
    ping(db)
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_bridge_by_path(
    db: &RedisDb,
    forest_uuid: &Uuid,
    path: String,
) -> CatlibResult<Arc<Mutex<dyn BridgeManifest>>> {
    let data = query_get(db, "bridge-*")?;

    let bridges: Vec<_> = data
        .into_iter()
        .map(|(key, value)| (key, BridgeData::from(value.as_ref().unwrap().as_str())))
        .filter(|(_key, bridge_data)| bridge_data.forest_uuid == *forest_uuid)
        .filter(|(_key, bridge_data)| bridge_data.path == PathBuf::from(path.clone()))
        .map(|(_key, data)| {
            Arc::new(Mutex::new(Bridge::from_bridge_data(data, db)))
                as Arc<Mutex<dyn BridgeManifest>>
        })
        .collect();

    match bridges.as_slice() {
        [] => Err(CatlibError::NoRecordsFound),
        [bridge] => Ok(bridge.clone()),
        _ => Err(CatlibError::MalformedDatabaseRecord(
            "More than 1 bridge found in Catalog Backend".into(),
        )),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_containers_by_forest_uuid(
    db: &RedisDb,
    forest_uuid: &Uuid,
) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>> {
    let data = query_get(db, "container-*")?;

    let containers: Vec<_> = data
        .iter()
        .filter(|(_, value)| value.is_some())
        .map(|(key, value)| {
            (
                key.clone(),
                ContainerData::from(value.as_ref().unwrap().as_str()),
            )
        })
        .filter(|(_key, data)| data.forest_uuid == *forest_uuid)
        .map(|(_key, data)| {
            Arc::new(Mutex::new(ContainerEntity::from_container_data(data, db)))
                as Arc<Mutex<dyn ContainerManifest>>
        })
        .collect();

    match containers.len() {
        0 => Err(CatlibError::NoRecordsFound),
        _ => Ok(containers),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_all_containers(
    db: &RedisDb,
    forest_uuid: &Uuid,
) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>> {
    let data = query_get(db, "container-*")?;

    let containers: Vec<_> = data
        .iter()
        .filter(|(_, value)| value.is_some())
        .map(|(key, value)| {
            (
                key.clone(),
                ContainerData::from(value.as_ref().unwrap().as_str()),
            )
        })
        .filter(|(_key, data)| data.forest_uuid == *forest_uuid)
        .map(|(_key, data)| {
            Arc::new(Mutex::new(ContainerEntity::from_container_data(data, db)))
                as Arc<Mutex<dyn ContainerManifest>>
        })
        .collect();

    match containers.len() {
        0 => Err(CatlibError::NoRecordsFound),
        _ => Ok(containers),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_containers_by_path(
    db: &RedisDb,
    forest_uuid: &Uuid,
    paths: ContainerPaths,
    include_subdirs: bool,
) -> CatlibResult<Vec<Arc<Mutex<dyn ContainerManifest>>>> {
    let data = query_get(db, "container-*")?;

    let containers: Vec<_> = data
        .iter()
        .filter(|(_, value)| value.is_some())
        .map(|(key, value)| {
            (
                key.clone(),
                ContainerData::from(value.as_ref().unwrap().as_str()),
            )
        })
        .filter(|(_key, data)| data.forest_uuid == *forest_uuid)
        .filter(|(_key, container_data)| {
            container_data.paths.iter().any(|container_path| {
                paths.iter().any(|path| {
                    (include_subdirs && container_path.starts_with(path))
                        || container_path.eq(&PathBuf::from(path))
                })
            })
        })
        .map(|(_key, data)| {
            Arc::new(Mutex::new(ContainerEntity::from_container_data(data, db)))
                as Arc<Mutex<dyn ContainerManifest>>
        })
        .collect();

    match containers.len() {
        0 => Err(CatlibError::NoRecordsFound),
        _ => Ok(containers),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_container_by_uuid(
    db: &RedisDb,
    uuid: &Uuid,
) -> CatlibResult<Arc<Mutex<dyn ContainerManifest>>> {
    let data = get(db, format!("container-{uuid}"))?;

    match data {
        Some(serialised) => {
            let data = ContainerData::from(serialised.as_str());

            Ok(
                Arc::new(Mutex::new(ContainerEntity::from_container_data(data, db)))
                    as Arc<Mutex<dyn ContainerManifest>>,
            )
        }
        None => Err(CatlibError::NoRecordsFound),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_forest_by_uuid(
    db: &RedisDb,
    uuid: &Uuid,
) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
    let data = get(db, format!("forest-{uuid}"))?;

    match data {
        Some(serialised) => {
            let data = ForestData::from(serialised.as_str());
            let forest = ForestEntity {
                data,
                db: db.clone(),
            };

            Ok(Arc::new(Mutex::new(forest)))
        }
        None => Err(CatlibError::NoRecordsFound),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_all_forests(db: &RedisDb) -> CatlibResult<Vec<Arc<Mutex<dyn ForestManifest>>>> {
    let data = query_get(db, "forest-*")?;

    let forests: Vec<_> = data
        .iter()
        .filter(|(_, value)| value.is_some())
        .map(|(key, value)| {
            (
                key.clone(),
                ForestData::from(value.as_ref().unwrap().as_str()),
            )
        })
        .map(|(_key, data)| {
            Arc::new(Mutex::new(ForestEntity::from_forest_data(data, db)))
                as Arc<Mutex<dyn ForestManifest>>
        })
        .collect();

    match forests.len() {
        0 => Err(CatlibError::NoRecordsFound),
        _ => Ok(forests),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_templates(db: &RedisDb) -> CatlibResult<impl Iterator<Item = (Uuid, String)>> {
    let data = query_get(db, "template-*")?;

    Ok(data.into_iter().filter_map(|(key, value)| {
        value.map(|v| {
            (
                Uuid::from_str(key.strip_prefix("template-").unwrap()).unwrap(),
                v,
            )
        })
    }))
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_storages_by_template_uuid(
    db: &RedisDb,
    template_uuid: &Uuid,
) -> CatlibResult<Vec<Arc<Mutex<dyn StorageManifest>>>> {
    let data = query_get(db, "storage-*")?;

    let storages: Vec<_> = data
        .iter()
        .filter(|(_, value)| value.is_some())
        .map(|(key, value)| (key.clone(), StorageData::from(value.as_ref().unwrap().as_str())))
        .filter(|(_key, storage_data)|
            matches!(storage_data.template_uuid, Some(val) if val == *template_uuid)
        )
        .map(|(_key, storage_data)| {
            Arc::new(Mutex::new(StorageEntity {
                storage_data,
                db: db.clone(),
            })) as Arc<Mutex<dyn StorageManifest>>
        })
        .collect();

    match storages.len() {
        0 => Err(CatlibError::NoRecordsFound),
        _ => Ok(storages),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_storages_by_container_uuid(
    db: &RedisDb,
    uuid: &Uuid,
) -> CatlibResult<Vec<Arc<Mutex<dyn StorageManifest>>>> {
    let data = query_get(db, "storage-*")?;

    let storages: Vec<_> = data
        .iter()
        .filter(|(_, value)| value.is_some())
        .map(|(key, value)| {
            (
                key.clone(),
                StorageData::from(value.as_ref().unwrap().as_str()),
            )
        })
        .filter(|(_key, storage_data)| storage_data.container_uuid == *uuid)
        .map(|(_key, storage_data)| {
            Arc::new(Mutex::new(StorageEntity {
                storage_data,
                db: db.clone(),
            })) as Arc<Mutex<dyn StorageManifest>>
        })
        .collect();

    match storages.len() {
        0 => Err(CatlibError::NoRecordsFound),
        _ => Ok(storages),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_storage_by_uuid(
    db: &RedisDb,
    uuid: &Uuid,
) -> CatlibResult<Arc<Mutex<dyn StorageManifest>>> {
    let data = get(db, format!("storage-{uuid}"))?;

    match data {
        Some(serialised) => {
            let data = StorageData::from(serialised.as_str());
            let storage = StorageEntity::from_storage_data(data, db);

            Ok(Arc::new(Mutex::new(storage)))
        }
        None => Err(CatlibError::NoRecordsFound),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn fetch_bridge_by_uuid(
    db: &RedisDb,
    uuid: &Uuid,
) -> CatlibResult<Arc<Mutex<dyn BridgeManifest>>> {
    let data = get(db, format!("bridge-{uuid}"))?;

    match data {
        Some(serialised) => {
            let data = BridgeData::from(serialised.as_str());
            let bridge = Bridge::from_bridge_data(data, db);

            Ok(Arc::new(Mutex::new(bridge)))
        }
        None => Err(CatlibError::NoRecordsFound),
    }
}

#[cfg(test)]
pub(crate) mod test {
    use rstest::{fixture, *};
    use wildland_corex::catlib_service::entities::{ForestManifest, Identity, Signers};
    use wildland_corex::catlib_service::interface::CatLib;

    use crate::*;

    #[fixture]
    pub fn catlib() -> crate::RedisCatLib {
        let random = rand::random::<uuid::Bytes>();
        let uuid = uuid::Builder::from_random_bytes(random).into_uuid();
        let redis_url =
            std::env::var("CARGO_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".into());
        crate::RedisCatLib::new(redis_url, Some(uuid.to_string()))
    }

    fn _make_forest_with_signer(catlib: &RedisCatLib) -> Arc<Mutex<dyn ForestManifest>> {
        let owner = Identity([1; 32]);
        let signer = Identity([2; 32]);

        let mut signers = Signers::new();
        signers.insert(signer);

        catlib.create_forest(owner, signers, vec![]).unwrap()
    }

    #[rstest]
    fn db_read_with_many_threads(catlib: RedisCatLib) {
        let run_fetch = |i: u32, db: RedisDb| async move {
            println!(
                "Hello from task {}, tid: {:?}",
                i,
                std::thread::current().id()
            );

            assert!(db::fetch_templates(&db).is_ok());
        };

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(8)
            .build()
            .unwrap();

        runtime.block_on(futures::future::join_all(
            (0..100).map(|i| runtime.spawn(run_fetch(i, catlib.db.clone()))),
        ));
    }
}
