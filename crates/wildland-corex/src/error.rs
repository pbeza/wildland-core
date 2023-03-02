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

use std::fmt::Display;

use thiserror::Error;
use wildland_crypto::error::CryptoError;

use crate::catlib_service::error::CatlibError;
use crate::LssError;

pub trait ErrContext<T, E> {
    fn context(self, ctx: impl Display) -> Result<T, E>;
    fn format(err: impl Display, ctx: impl Display) -> String {
        format!("{ctx}: {err}")
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum ForestRetrievalError {
    #[error(transparent)]
    LssError(#[from] LssError),
    #[error("Could not create keypair from bytes retrieved from LSS: {0}")]
    KeypairParseError(CryptoError),
}

#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum CoreXError {
    #[error("Cannot create forest identity: {0}")]
    CannotCreateForestIdentityError(String),
    #[error("Identity read error: {0}")]
    IdentityReadError(String),
    #[error("LSS Error: {0}")]
    LSSErr(String, LssError),
    #[error("Catlib Error: {0}: {1}")]
    CatlibErr(String, CatlibError),
    #[error("Crypto Error: {0}: {1}")]
    CryptoErr(String, CryptoError),
    #[error("CoreX error: {0}")]
    Generic(String),
}

impl<T> ErrContext<T, CoreXError> for Result<T, CatlibError> {
    fn context(self, ctx: impl Display) -> Result<T, CoreXError> {
        self.map_err(|e| CoreXError::CatlibErr(ctx.to_string(), e))
    }
}

impl<T> ErrContext<T, CoreXError> for Result<T, CryptoError> {
    fn context(self, ctx: impl Display) -> Result<T, CoreXError> {
        self.map_err(|e| CoreXError::CryptoErr(ctx.to_string(), e))
    }
}

impl<T> ErrContext<T, CoreXError> for Result<T, LssError> {
    fn context(self, ctx: impl Display) -> Result<T, CoreXError> {
        self.map_err(|e| CoreXError::LSSErr(ctx.to_string(), e))
    }
}
