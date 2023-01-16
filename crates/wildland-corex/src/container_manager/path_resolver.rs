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

use uuid::Uuid;

use crate::Storage;

/// Represents result of a possible path within a Storage. Storages field represents all alternative
/// locations of the path.
///
#[derive(Debug)]
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

    /// Lists all virtual nodes' names contained by provided path.
    /// Example: C1 claims /a/b, C2 claims /a/c
    ///     when called with arg `/a` returns ["b", "c"]
    fn list_virtual_nodes_in(&self, path: &Path) -> Vec<String>;

    /// Checks if provided path is a virtual node
    fn is_virtual_nodes(&self, path: &Path) -> bool;
}
