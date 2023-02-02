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

pub(crate) use wildland_corex::catlib_service::error::{CatlibError, CatlibResult};

pub(crate) fn redis_to_catlib_err(err: redis::RedisError) -> CatlibError {
    CatlibError::Generic(format!("Redis error: {err}"))
}

pub(crate) fn r2d2_to_catlib_err(err: r2d2::Error) -> CatlibError {
    CatlibError::Generic(format!("R2D2 error: {err}"))
}
