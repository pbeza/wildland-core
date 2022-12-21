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

use crate::unencrypted::UnencryptedDfs;
use wildland_corex::dfs::interfaces::{Dfs as IDfs, DfsFrontend};

pub struct EncryptedDfs {
    inner: UnencryptedDfs,
}

impl EncryptedDfs {
    pub fn new() -> Self {
        Self {
            inner: UnencryptedDfs {},
        }
    }
}

impl IDfs for EncryptedDfs {
    fn get_version(&self) -> &'static str {
        self.inner.get_version()
    }

    fn init_storage_driver(&self) -> Result<(), wildland_corex::dfs::error::DfsError> {
        todo!()
    }
}

impl DfsFrontend for EncryptedDfs {
    fn readdir(&self) {
        // TODO encrypt/decrypt and delegate to unencrypted dfs
        self.inner.readdir()
    }
}
