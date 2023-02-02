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

use std::rc::Rc;

use thiserror::Error;
use wildland_crypto::error::CryptoError;

use crate::cross_platform_http_client::HttpError;

#[derive(Error, Debug, Clone)]
#[repr(C)]
pub enum WildlandHttpClientError {
    #[error("Application HTTP Error: {0}")]
    ApplicationHttpError(Rc<anyhow::Error>),
    #[error("Client HTTP Error: {0}")]
    ClientHttpError(#[from] HttpError),
    #[error(transparent)]
    CommonLibError(#[from] CryptoError),
}

impl From<http::Error> for WildlandHttpClientError {
    fn from(value: http::Error) -> Self {
        Self::ApplicationHttpError(Rc::new(value.into()))
    }
}

impl From<serde_json::Error> for WildlandHttpClientError {
    fn from(value: serde_json::Error) -> Self {
        Self::ApplicationHttpError(Rc::new(value.into()))
    }
}
