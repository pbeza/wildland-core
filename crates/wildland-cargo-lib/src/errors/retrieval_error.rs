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

pub type RetrievalResult<T, E> = Result<T, RetrievalError<E>>;
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum RetrievalError<E: Clone> {
    NotFound(String),
    Unexpected(E),
}

impl<E: Display + Clone> ExceptionTrait for RetrievalError<E> {
    fn reason(&self) -> String {
        match self {
            RetrievalError::NotFound(s) => s.to_string(),
            RetrievalError::Unexpected(e) => e.to_string(),
        }
    }
}
