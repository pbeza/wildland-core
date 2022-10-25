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

use std::rc::Rc;

use http::StatusCode;
use minreq::Response;

use crate::error::WildlandHttpClientError::{self, HttpError};

#[tracing::instrument(level = "debug", ret)]
pub(crate) fn check_status_code(response: Response) -> Result<Response, WildlandHttpClientError> {
    match StatusCode::from_u16(response.status_code as u16)
        .map_err(|e| WildlandHttpClientError::HttpError(e.to_string()))?
    {
        StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED | StatusCode::NO_CONTENT => {
            Ok(response)
        }
        _ => Err(HttpError(response.as_str().map_err(Rc::new)?.to_owned())),
    }
}
