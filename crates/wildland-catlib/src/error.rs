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

use rustbreak::error::RustbreakError;
pub(crate) use wildland_corex::catlib_service::error::{CatlibError, CatlibResult};

pub(crate) fn to_catlib_error(rb_error: RustbreakError) -> CatlibError {
    match rb_error {
        RustbreakError::DeSerialization(_) => {
            CatlibError::Generic("RustbreakError::DeSerialization".into())
        }
        RustbreakError::Poison => CatlibError::Generic("RustbreakError::Poison".into()),
        RustbreakError::Backend(_) => CatlibError::Generic("RustbreakError::Backend".into()),
        RustbreakError::WritePanic => CatlibError::Generic("RustbreakError::WritePanic".into()),
        _ => CatlibError::Generic("Unknown Rustbreak error".into()),
    }
}
