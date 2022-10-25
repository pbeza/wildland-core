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

mod retrieval_error;
mod single_variant;
mod user;
pub use retrieval_error::*;
pub use single_variant::*;
pub use user::*;

use crate::api::foundation_storage::FsaError;

impl ExceptionTrait for FsaError {
    fn reason(&self) -> String {
        self.to_string()
    }
}

pub trait ExceptionTrait {
    fn reason(&self) -> String;
}
