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

use wildland_corex::storage::StorageTemplate as InnerStorageTemplate;

#[derive(Debug, Clone)]
pub struct StorageTemplate {
    inner: InnerStorageTemplate,
}

impl StorageTemplate {
    pub(crate) fn new(inner: InnerStorageTemplate) -> Self {
        Self { inner }
    }

    pub(crate) fn inner(&self) -> &InnerStorageTemplate {
        &self.inner
    }

    pub fn stringify(&self) -> String {
        format!("Storage Template (uuid: {})", self.inner.uuid())
    }
}
