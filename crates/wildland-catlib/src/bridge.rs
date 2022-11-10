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

use super::*;
use serde::{Deserialize, Serialize};
use std::{rc::Rc, str::FromStr};
use wildland_corex::entities::{Bridge as IBridge, BridgeData, ContainerPath, Forest};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Bridge {
    data: BridgeData,

    #[serde(skip, default = "use_default_database")]
    db: Rc<StoreDb>,
}

/// Create Bridge object from its representation in Rust Object Notation
impl FromStr for Bridge {
    type Err = ron::error::SpannedError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ron::from_str(value)
    }
}

impl AsRef<BridgeData> for Bridge {
    fn as_ref(&self) -> &BridgeData {
        &self.data
    }
}

impl Bridge {
    pub fn new(forest_uuid: Uuid, path: ContainerPath, link: Vec<u8>, db: Rc<StoreDb>) -> Self {
        Bridge {
            data: BridgeData {
                uuid: Uuid::new_v4(),
                forest_uuid,
                path,
                link,
            },
            db,
        }
    }
}

impl IBridge for Bridge {
    /// ## Errors
    ///
    /// - Returns [`CatlibError::NoRecordsFound`] if no [`Forest`] was found.
    /// - Returns [`CatlibError::MalformedDatabaseRecord`] if more than one [`Forest`] was found.
    fn forest(&self) -> CatlibResult<Box<dyn Forest>> {
        fetch_forest_by_uuid(self.db.clone(), &self.data.forest_uuid)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn update(&mut self, link: Vec<u8>) -> CatlibResult<&mut dyn IBridge> {
        self.data.link = link;
        self.save()?;
        Ok(self)
    }

    /// ## Errors
    ///
    /// Returns `RustbreakError` cast on [`CatlibResult`] upon failure to save to the database.
    fn delete(&mut self) -> CatlibResult<bool> {
        Model::delete(self)?;
        Ok(true)
    }
}

impl Model for Bridge {
    fn save(&mut self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("bridge-{}", self.data.uuid),
            ron::to_string(self).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(self.db.clone(), format!("bridge-{}", self.data.uuid))
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
            .create_forest(Identity([1; 32]), Signers::new(), vec![])
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
