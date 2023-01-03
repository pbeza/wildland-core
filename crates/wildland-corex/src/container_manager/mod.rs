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

mod path_resolver;

pub use path_resolver::*;
use std::path::Path;

pub struct ContainerManager;

impl PathResolver for ContainerManager {
    fn resolve(&self, _path: &Path) -> Vec<PathWithStorages> {
        todo!() // TODO WILX-353 implement when ContainerManager is filled with information about mounted containers
    }
}
