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

use std::fmt::Display;

pub enum StorageBackendType {
    FoundationStorage,
    LocalFilesystem,
    Custom(String),
}

impl Display for StorageBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageBackendType::FoundationStorage => write!(f, "FoundationStorage"),
            StorageBackendType::LocalFilesystem => write!(f, "LocalFilesystem"),
            StorageBackendType::Custom(t) => write!(f, "{t}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Storage {}

impl Storage {
    pub fn stringify(&self) -> String {
        todo!()
    }
}
