//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use rustbreak::{deser::Ron, PathDatabase};

use super::CatlibResult;

pub type Identity = Vec<u8>;
pub(crate) type CatLibData = std::collections::HashMap<String, String>;
pub(crate) type StoreDb = PathDatabase<CatLibData, Ron>;

pub(crate) trait Model {
    fn delete(&mut self) -> CatlibResult<()>;
    fn save(&mut self) -> CatlibResult<()>;
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
