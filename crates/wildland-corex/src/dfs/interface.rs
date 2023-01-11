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

#[derive(Debug, PartialEq, Eq)]
pub struct Stat {
    pub node_type: NodeType,
    // TODO fields?
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeType {
    File,
    Dir,
    Symlink,
}

/// Interface that DFS should expose towards frontends
pub trait DfsFrontend {
    fn readdir(&mut self, path: String) -> Vec<String>;
    fn getattr(&mut self, path: String) -> Option<Stat>;
}
