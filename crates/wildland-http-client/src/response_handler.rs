use std::rc::Rc;

use http::StatusCode;
use minreq::Response;

use crate::error::WildlandHttpClientError::{self, HttpError};

#[tracing::instrument(level = "debug", ret)]
pub(crate) fn handle(response: Response) -> Result<Option<Response>, WildlandHttpClientError> {
    match StatusCode::from_u16(response.status_code as u16)
        .map_err(|e| WildlandHttpClientError::HttpError(e.to_string()))?
    {
        StatusCode::OK => Ok(Some(response)),
        StatusCode::CREATED | StatusCode::ACCEPTED | StatusCode::NO_CONTENT => Ok(None),
        _ => Err(HttpError(response.as_str().map_err(Rc::new)?.to_owned())),
    }
}
