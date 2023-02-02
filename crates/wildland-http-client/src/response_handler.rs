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

use anyhow::anyhow;
use http::StatusCode;

use super::cross_platform_http_client::Response;
use crate::error::WildlandHttpClientError;

pub(crate) fn check_status_code(response: Response) -> Result<Response, WildlandHttpClientError> {
    match response.status() {
        StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED | StatusCode::NO_CONTENT => {
            Ok(response)
        }
        v => Err(WildlandHttpClientError::ApplicationHttpError(Rc::new(
            anyhow!(
                "HTTP response code: {v}; payload: {}",
                String::from_utf8_lossy(response.body())
            ),
        ))),
    }
}
