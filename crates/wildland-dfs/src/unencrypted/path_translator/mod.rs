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

use super::NodeDescriptor;

pub mod uuid_in_dir;

pub trait PathConflictResolver {
    // Returns paths under which the nodes are to be exposed
    fn solve_conflicts<'a>(
        &self,
        nodes: Vec<&'a NodeDescriptor>,
    ) -> Vec<(&'a NodeDescriptor, PathBuf)>;

    /// determines absolute path basing on the exposed one
    fn exposed_to_absolute_path(&self, path: &Path) -> PathBuf;
}
