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

/// Create String object from its representation in Rust Object Notation
impl TryFrom<String> for Storage {
    type Error = ron::error::SpannedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ron::from_str(value.as_str())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Storage {
    uuid: Uuid,
    container_uuid: Uuid,
    template_uuid: Option<Uuid>,
    data: Vec<u8>,

    #[serde(skip, default = "use_default_database")]
    db: Rc<StoreDb>,
}

impl Storage {
    pub fn new(
        container_uuid: Uuid,
        template_uuid: Option<Uuid>,
        data: Vec<u8>,
        db: Rc<StoreDb>,
    ) -> Self {
        Storage {
            uuid: Uuid::new_v4(),
            container_uuid,
            template_uuid,
            data,
            db,
        }
    }
}

impl IStorage for Storage {
    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn template_uuid(&self) -> Option<Uuid> {
        self.template_uuid
    }

    fn container(&self) -> CatlibResult<crate::container::Container> {
        fetch_container_by_uuid(self.db.clone(), self.container_uuid)
    }

    fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn update(&mut self, data: Vec<u8>) -> CatlibResult<crate::storage::Storage> {
        self.data = data;
        self.save()?;
        Ok(self.clone())
    }
}

impl Model for Storage {
    fn save(&mut self) -> CatlibResult<()> {
        save_model(
            self.db.clone(),
            format!("storage-{}", self.uuid()),
            ron::to_string(self).unwrap(),
        )
    }

    fn delete(&mut self) -> CatlibResult<()> {
        delete_model(self.db.clone(), format!("storage-{}", self.uuid()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::container::Container;
    use crate::contracts::IStorage;
    use crate::storage::Storage;
    use crate::*;
    use rstest::*;
    use uuid::Bytes;

    fn _catlib() -> CatLib {
        let catlib = db::init_catlib(rand::random::<Bytes>());

        // Create a dummy forest and container to which storages will be bound
        catlib
            .create_forest(
                Identity([1; 32]),
                HashSet::from([Identity([2; 32])]),
                vec![],
            )
            .unwrap();

        catlib
    }

    fn _container(catlib: &CatLib) -> Container {
        let forest = catlib.find_forest(Identity([1; 32])).unwrap();
        forest.create_container().unwrap()
    }

    #[fixture]
    fn catlib() -> CatLib {
        _catlib()
    }

    #[fixture]
    fn container() -> Container {
        let catlib = _catlib();
        _container(&catlib)
    }

    fn make_storage(container: &Container) -> Storage {
        container.create_storage(None, vec![]).unwrap()
    }

    fn make_storage_with_template(container: &Container, template_id: Uuid) -> Storage {
        container.create_storage(Some(template_id), vec![]).unwrap()
    }

    #[rstest]
    fn create_empty_storage(container: Container) {
        make_storage(&container);

        assert_eq!(container.storages().unwrap().len(), 1);

        make_storage(&container);

        assert_eq!(container.storages().unwrap().len(), 2);
    }

    #[rstest]
    fn delete_a_storage(container: Container) {
        make_storage(&container);
        let mut storage = make_storage(&container);

        storage.delete().unwrap();

        assert_eq!(container.storages().unwrap().len(), 1);
    }

    #[rstest]
    fn create_storage_with_template_id(catlib: CatLib) {
        let container = _container(&catlib);
        make_storage(&container); // Create storage w/o template id on purpose
        make_storage_with_template(&container, Uuid::from_u128(1));
        make_storage_with_template(&container, Uuid::from_u128(1));
        make_storage_with_template(&container, Uuid::from_u128(2));

        let storages = catlib
            .find_storages_with_template(Uuid::from_u128(1))
            .unwrap();
        assert_eq!(storages.len(), 2);

        let storages = catlib
            .find_storages_with_template(Uuid::from_u128(2))
            .unwrap();
        assert_eq!(storages.len(), 1);
    }

    #[rstest]
    fn create_storage_with_data(container: Container) {
        container
            .create_storage(None, b"storage data".to_vec())
            .unwrap();

        assert_eq!(container.storages().unwrap()[0].data(), b"storage data")
    }
}
