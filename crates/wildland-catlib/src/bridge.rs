//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

impl TryFrom<String> for Bridge {
    type Error = ron::error::SpannedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ron::from_str(value.as_str())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Bridge {
    uuid: String,
    forest_uuid: String,
    path: ContainerPath,
    link: Vec<u8>,

    #[serde(skip, default = "use_default_database")]
    db: Rc<StoreDb>,
}

impl Bridge {
    pub fn new(forest_uuid: String, path: ContainerPath, link: Vec<u8>, db: Rc<StoreDb>) -> Self {
        Bridge {
            uuid: Uuid::new_v4().to_string(),
            forest_uuid,
            path,
            link,
            db,
        }
    }
}

impl IBridge for Bridge {
    fn uuid(&self) -> String {
        self.uuid.clone()
    }

    fn path(&self) -> ContainerPath {
        self.path.clone()
    }

    fn forest(&self) -> CatlibResult<crate::forest::Forest> {
        fetch_forest_by_uuid(self.db.clone(), self.forest_uuid.clone())
    }

    fn link(&self) -> Vec<u8> {
        self.link.clone()
    }

    fn update(&mut self, link: Vec<u8>) -> CatlibResult<crate::bridge::Bridge> {
        self.link = link;
        self.save()?;
        Ok(self.clone())
    }
}

impl Model for Bridge {
    fn save(&mut self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("bridge-{}", self.uuid),
            ron::to_string(self).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(self.db.clone(), format!("bridge-{}", self.uuid))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use rstest::*;
    use uuid::Bytes;

    #[fixture]
    fn catlib() -> CatLib {
        db::init_catlib(rand::random::<Bytes>())
    }

    #[rstest]
    fn create_bridge(catlib: CatLib) {
        let forest = catlib
            .create_forest(b"owner".to_vec(), Signers::new(), vec![])
            .unwrap();

        let mut bridge = forest
            .create_bridge("/other/forest".to_string(), vec![])
            .unwrap();

        forest.find_bridge("/other/forest".to_string()).unwrap();

        bridge.delete().unwrap();

        let bridge = forest.find_bridge("/other/forest".to_string());

        assert_eq!(bridge.err(), Some(CatlibError::NoRecordsFound));
    }
}
