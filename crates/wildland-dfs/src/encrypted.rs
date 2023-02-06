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

    fn create_dir(&mut self, path: String) -> Result<(), DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.create_dir(path)
    }

    fn remove_dir(&mut self, path: String) -> Result<(), DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.remove_dir(path)
    }

    fn read(&mut self, file: &FileHandle, count: usize) -> Result<Vec<u8>, DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.read(file, count)
    }

    fn write(&mut self, file: &FileHandle, buf: Vec<u8>) -> Result<usize, DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.write(file, buf)
    }

    fn seek_from_start(
        &mut self,
        file: &FileHandle,
        pos_from_start: usize,
    ) -> Result<usize, DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.seek_from_start(file, pos_from_start)
    }

    fn seek_from_current(
        &mut self,
        file: &FileHandle,
        pos_from_current: isize,
    ) -> Result<usize, DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.seek_from_current(file, pos_from_current)
    }

    fn seek_from_end(
        &mut self,
        file: &FileHandle,
        pos_from_end: usize,
    ) -> Result<usize, DfsFrontendError> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.seek_from_end(file, pos_from_end)
    }
}
