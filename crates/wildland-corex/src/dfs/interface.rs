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

use std::path::PathBuf;

use crate::Storage;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeDescriptor {
    // None value represents virtual nodes - parts of containers claimed paths
    pub storage: Option<NodeStorage>,
    pub absolute_path: PathBuf,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeStorage {
    storage: Storage,
    path_within_storage: PathBuf,
}

impl NodeStorage {
    pub fn new(storage: Storage, path_within_storage: PathBuf) -> Self {
        Self {
            storage,
            path_within_storage,
        }
    }
}

/// Interface that DFS should expose towards frontends
pub trait DfsFrontend {
    fn readdir(&mut self, path: String) -> Vec<NodeDescriptor>;
}
