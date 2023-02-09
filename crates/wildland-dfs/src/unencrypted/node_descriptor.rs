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

use std::path::{Component, Path, PathBuf};

use uuid::Uuid;
use wildland_corex::Storage;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeDescriptor {
    Physical {
        storages: NodeStorages,
        absolute_path: PathBuf,
    },
    Virtual {
        absolute_path: PathBuf,
    },
}

impl NodeDescriptor {
    pub fn abs_path(&self) -> &Path {
        match self {
            NodeDescriptor::Physical { absolute_path, .. }
            | NodeDescriptor::Virtual { absolute_path } => absolute_path,
        }
    }

    pub fn is_physical(&self) -> bool {
        match self {
            NodeDescriptor::Virtual { .. } => false,
            NodeDescriptor::Physical { .. } => true,
        }
    }

    pub fn is_virtual(&self) -> bool {
        !self.is_physical()
    }

    pub fn storages(&self) -> Option<&NodeStorages> {
        match self {
            NodeDescriptor::Physical { storages, .. } => Some(storages),
            NodeDescriptor::Virtual { .. } => None,
        }
    }

    pub fn parent(&self) -> Option<Self> {
        match self {
            NodeDescriptor::Physical {
                absolute_path,
                storages: node_storages,
            } => match node_storages.path_within_storage.components().last()? {
                Component::Normal(_) => Some(Self::Physical {
                    storages: NodeStorages {
                        storages: node_storages.storages.clone(),
                        path_within_storage: node_storages
                            .path_within_storage
                            .parent()
                            .map(|p| p.into())?,
                        uuid: node_storages.uuid,
                    },
                    absolute_path: absolute_path.parent().map(|p| p.into())?,
                }),
                _ => None,
            },
            NodeDescriptor::Virtual { absolute_path } => {
                absolute_path.parent().map(|p| Self::Virtual {
                    absolute_path: p.into(),
                })
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeStorages {
    storages: Vec<Storage>,
    path_within_storage: PathBuf,
    uuid: Uuid,
}

impl NodeStorages {
    pub fn new(storages: Vec<Storage>, path_within_storage: PathBuf, uuid: Uuid) -> Self {
        Self {
            storages,
            path_within_storage,
            uuid,
        }
    }

    pub fn path_within_storage(&self) -> &Path {
        &self.path_within_storage
    }

    pub fn storages(&self) -> &[Storage] {
        &self.storages
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
}
