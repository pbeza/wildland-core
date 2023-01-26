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

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use wildland_corex::catlib_service::entities::{ContainerPath, ForestManifest};
use wildland_corex::BridgeManifest;

use super::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BridgeData {
    pub uuid: Uuid,
    pub forest_uuid: Uuid,
    pub path: ContainerPath,
    pub link: Vec<u8>,
}

impl From<&str> for BridgeData {
    fn from(data_str: &str) -> Self {
        ron::from_str(data_str).unwrap()
    }
}

impl From<&Bridge> for String {
    fn from(value: &Bridge) -> Self {
        ron::to_string(&value.bridge_data).unwrap()
    }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub(crate) struct Bridge {
    pub(crate) bridge_data: BridgeData,

    #[derivative(Debug = "ignore")]
    pub(crate) db: RedisDb,
}

impl Bridge {
    pub fn new(forest_uuid: Uuid, path: ContainerPath, link: Vec<u8>, db: RedisDb) -> Self {
        Self {
            bridge_data: BridgeData {
                uuid: Uuid::new_v4(),
                forest_uuid,
                path,
                link,
            },
            db,
        }
    }

    pub fn from_bridge_data(bridge_data: BridgeData, db: RedisDb) -> Self {
        Self { bridge_data, db }
    }
}

impl BridgeManifest for Bridge {
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Forest`] was found.
    fn forest(&self) -> CatlibResult<Arc<Mutex<dyn ForestManifest>>> {
        fetch_forest_by_uuid(self.db.clone(), &self.bridge_data.forest_uuid)
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    fn update(&mut self, link: Vec<u8>) -> CatlibResult<()> {
        self.bridge_data.link = link;
        self.save()?;
        Ok(())
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to save to the database.
    fn remove(&mut self) -> CatlibResult<bool> {
        Model::delete(self)?;
        Ok(true)
    }

    /// ## Errors
    ///
    /// Returns `RedisError` cast on [`CatlibResult`] upon failure to read from the database.
    fn path(&mut self) -> CatlibResult<String> {
        self.sync()?;
        Ok(self.bridge_data.path.to_string_lossy().to_string())
    }

    fn uuid(&self) -> Uuid {
        self.bridge_data.uuid
    }

    fn serialise(&self) -> String {
        self.into()
    }
}

impl Model for Bridge {
    fn save(&self) -> CatlibResult<()> {
        db::commands::set(
            self.db.clone(),
            format!("bridge-{}", self.uuid()),
            self.serialise(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        db::commands::delete(self.db.clone(), format!("bridge-{}", self.uuid()))
    }

    fn sync(&mut self) -> CatlibResult<()> {
        let bridge = db::fetch_bridge_by_uuid(self.db.clone(), &self.uuid())?;
        let bridge_lock = bridge.lock().expect("Poisoned Mutex");
        self.bridge_data = BridgeData::from(bridge_lock.serialise().as_str());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::db::test::catlib;
    use crate::*;

    #[rstest]
    fn create_bridge(catlib: CatLib) {
        let forest = catlib
            .create_forest(Identity([1; 32]), Signers::new(), vec![])
            .unwrap();

        let bridge = forest
            .lock()
            .unwrap()
            .create_bridge("/other/forest".into(), vec![])
            .unwrap();

        forest
            .lock()
            .unwrap()
            .find_bridge("/other/forest".into())
            .unwrap();

        bridge.lock().unwrap().remove().unwrap();

        let bridge = forest.lock().unwrap().find_bridge("/other/forest".into());

        assert_eq!(bridge.err(), Some(CatlibError::NoRecordsFound));
    }
}
