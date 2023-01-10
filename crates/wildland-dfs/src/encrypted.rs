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

use std::collections::HashMap;
use std::rc::Rc;

use wildland_corex::dfs::interface::{DfsFrontend, NodeDescriptor};
use wildland_corex::PathResolver;

use crate::unencrypted::{StorageBackendFactory, UnencryptedDfs};

pub struct EncryptedDfs {
    inner: UnencryptedDfs,
}

impl EncryptedDfs {
    pub fn new(
        path_resolver: Rc<dyn PathResolver>,
        storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    ) -> Self {
        Self {
            inner: UnencryptedDfs::new(path_resolver, storage_backend_factories),
        }
    }
}

impl DfsFrontend for EncryptedDfs {
    fn readdir(&mut self, path: String) -> Vec<NodeDescriptor> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.readdir(path)
    }
}
