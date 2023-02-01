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

use wildland_corex::dfs::interface::{DfsFrontend, DfsFrontendError, FileHandle, Stat};
use wildland_corex::PathResolver;

use crate::storage_backends::StorageBackendFactory;
use crate::unencrypted::UnencryptedDfs;

pub struct EncryptedDfs {
    inner: UnencryptedDfs,
}

impl EncryptedDfs {
    pub fn new(
        path_resolver: Box<dyn PathResolver>,
        storage_backend_factories: HashMap<String, Box<dyn StorageBackendFactory>>,
    ) -> Self {
        Self {
            inner: UnencryptedDfs::new(path_resolver, storage_backend_factories),
        }
    }
}

impl DfsFrontend for EncryptedDfs {
    fn readdir(&mut self, path: String) -> Result<Vec<String>, DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.readdir(path)
    }

    fn getattr(&mut self, path: String) -> Result<Stat, DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.getattr(path)
    }

    fn open(&mut self, path: String) -> Result<FileHandle, DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.open(path)
    }

    fn close(&mut self, file: &FileHandle) -> Result<(), DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.close(file)
    }
}
