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

use std::path::{Path, PathBuf};

use crate::Storage;

/// Represents result of a possible path within a Storage. Storages field represents all alternative
/// locations of the path.
///
pub enum ResolvedPath {
    PathWithStorages {
        /// path within storages
        path_within_storage: PathBuf,
        /// all storages that include the path (all replicas)
        storages: Vec<Storage>,
    },
    /// represents virtual node paths that are not supposed to be looked up for in any backend
    VirtualPath(PathBuf),
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
    /// Storages from different containers are represented bu different elements in a resulting vector
    /// because the matching paths inside containers may be different. E.g. if container C1 claims path
    /// `/a/` and container C2 claims path `/a/b/` then when PathResolver is asked about path `/a/b/c`
    /// two-element vector is returned:
    /// [
    ///     PathWithStorages { path: "/b/c/", storages: [all storages of C1]},
    ///     PathWithStorages { path: "/c/", storages: [all storages of C2]},
    /// ]
    ///
    fn resolve(&self, path: &Path) -> Vec<ResolvedPath>;
}
