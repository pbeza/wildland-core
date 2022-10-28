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

pub mod container;
pub mod retrieval_error;
pub mod single_variant;
pub mod storage;
pub mod user;

use crate::api::foundation_storage::FsaError;

impl ExceptionTrait for FsaError {
    fn reason(&self) -> String {
        self.to_string()
    }
}

pub trait ExceptionTrait {
    fn reason(&self) -> String;
}
