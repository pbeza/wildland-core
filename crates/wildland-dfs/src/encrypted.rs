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

use std::{path::Path, rc::Rc};

use crate::unencrypted::UnencryptedDfs;
use wildland_corex::{
    dfs::interface::{DfsFrontend, NodeDescriptor},
    PathResolver,
};

pub struct EncryptedDfs {
    inner: UnencryptedDfs,
}

impl EncryptedDfs {
    pub fn new(path_resolver: Rc<dyn PathResolver>) -> Self {
        Self {
            inner: UnencryptedDfs::new(path_resolver),
        }
    }
}

impl DfsFrontend for EncryptedDfs {
    fn readdir<P: AsRef<Path>>(&mut self, path: P) -> Vec<NodeDescriptor> {
        // TODO WILX-11 encrypt/decrypt and delegate to unencrypted dfs
        self.inner.readdir(path)
    }
}
