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

use crate::Storage;
use std::path::{Path, PathBuf};

/// Represents result of a possible path within a Storage.
///
pub struct PathWithinStorage {
    pub path: PathBuf,
    pub storages: Vec<Storage>,
}

#[mockall::automock]
pub trait PathResolver {
    /// Returns Storages of containers claiming paths that match the provided argument along with
    /// the part of a path that is inside the container.
    ///
    /// **Example**: if a container claims path `/a/b/` and [`PathResolver`] receives request to resolve
    /// path `/a/b/c/d` then [`PathResolver`] should return path `/c/d` with all Storages of that
    /// container, so DFS could choose which Storage to use.
    ///
    fn resolve(&self, path: &Path) -> Vec<PathWithinStorage>;
}
