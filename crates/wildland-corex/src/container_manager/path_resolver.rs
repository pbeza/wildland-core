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

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

use crate::catlib_service::error::CatlibError;
use crate::Storage;

/// Represents result of a possible path within a Storage. Storages field represents all alternative
/// locations of the path.
///
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ResolvedPath {
    PathWithStorages {
        /// path within storages
        path_within_storage: PathBuf,
        /// Container uuid may be used but DFS itself does not recognize Container notion so it is called StoragesId
        storages_id: Uuid,
        /// all storages that include the path (all replicas)
        storages: Vec<Storage>,
    },
    /// Represents virtual node path that are not supposed to be looked up for in any backend.
    /// Contains absolute path in forest namespace.
    VirtualPath(PathBuf),
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum PathResolutionError {
    #[error(transparent)]
    CatlibError(#[from] CatlibError),
    #[error("Resolution error: {0}")]
    Generic(String),
}

#[mockall::automock]
pub trait PathResolver {
    /// Returns Storages of containers claiming paths that match the provided argument along with
    /// the part of a path that is inside the container.
    ///
    /// **Example**: if a container claims path `/a/b/` and [`PathResolver`] receives request to resolve
    /// path `/a/b/c/d` then [`PathResolver`] should return path `/c/d` with all Storages of that
    /// container as a single [`PathWithStorages`] instance, so DFS could choose which Storage to use.
    ///
    /// Storages from different containers are represented by different elements in a resulting vector
    /// because the matching paths inside containers may be different. Additionally, method returns full
    /// paths of containers that starts with the provided one as an argument.
    ///
    /// E.g. if container C1 claims path `/a/` and container C2 claims path `/a/b/` and container C3 claims
    /// path `/a/b/c/d` when PathResolver is asked about path `/a/b/c`, then three-element vector is returned:
    /// [
    ///     PathWithStorages { path: "/b/c/", storages: [all storages of C1]},
    ///     PathWithStorages { path: "/c/", storages: [all storages of C2]},
    ///     VirtualPath ( "/a/b/c/d" ),
    /// ]
    ///
    ///
    fn resolve(&self, path: &Path) -> Result<HashSet<ResolvedPath>, PathResolutionError>; // TODO it can return iterator
}
