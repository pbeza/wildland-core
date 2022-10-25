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

use std::rc::Rc;

use uuid::Uuid;

pub trait StorageTemplateTrait {
    fn uuid(&self) -> Uuid;
    fn data(&self) -> Vec<u8>;
}

#[derive(Clone)]
pub struct StorageTemplate {
    inner: Rc<dyn StorageTemplateTrait>,
}

impl StorageTemplate {
    pub fn uuid(&self) -> Uuid {
        self.inner.uuid()
    }

    pub fn data(&self) -> Vec<u8> {
        self.inner.data()
    }

    pub fn with_template(storage_template: Rc<dyn StorageTemplateTrait>) -> Self {
        Self {
            inner: storage_template,
        }
    }
}
