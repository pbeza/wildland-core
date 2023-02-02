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

use wildland_corex::catlib_service::error::CatlibResult;

pub(crate) type DbClient = r2d2::Pool<redis::Client>;

#[derive(Clone)]
pub(crate) struct RedisDb {
    pub client: DbClient,
    pub key_prefix: String,
}

pub trait Model {
    fn delete(&mut self) -> CatlibResult<()>;
    fn save(&self) -> CatlibResult<()>;
    fn sync(&mut self) -> CatlibResult<()>;
}

pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
