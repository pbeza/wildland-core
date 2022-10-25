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

use super::ExceptionTrait;
use std::fmt::Display;

pub type SingleErrVariantResult<T, E> = Result<T, SingleVariantError<E>>;
#[derive(Debug, Clone)]
#[repr(C)]
pub enum SingleVariantError<T: Clone> {
    Failure(T),
}

impl<E: Display + Clone> ExceptionTrait for SingleVariantError<E> {
    fn reason(&self) -> String {
        match self {
            Self::Failure(e) => e.to_string(),
        }
    }
}
