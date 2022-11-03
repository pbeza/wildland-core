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

#[derive(Error, Debug, Clone)]
pub enum WildlandHttpClientError {
    #[error("{0}")]
    HttpError(String),
    #[error("Cannot serialize request - source: {0}")]
    CannotSerializeRequestError(Rc<serde_json::Error>),
    #[error(transparent)]
    CommonLibError(#[from] CryptoError),
    #[error(transparent)]
    HttpLibError(#[from] Rc<minreq::Error>),
}
