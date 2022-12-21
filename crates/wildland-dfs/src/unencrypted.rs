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

use wildland_corex::dfs::interfaces::{Dfs as IDfs, DfsFrontend};

pub struct UnencryptedDfs {}

impl IDfs for UnencryptedDfs {
    fn get_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn init_storage_driver(&self) -> Result<(), wildland_corex::dfs::error::DfsError> {
        todo!()
    }
}

impl DfsFrontend for UnencryptedDfs {
    fn readdir(&self) {}
}
