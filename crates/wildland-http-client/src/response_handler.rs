use std::sync::Arc;

use crate::error::WildlandHttpClientError;
use reqwest::{Response, StatusCode};

use crate::error::WildlandHttpClientError::HttpError;

#[tracing::instrument(level = "debug", ret)]
pub(crate) async fn handle(
    response: Response,
) -> Result<Option<Response>, WildlandHttpClientError> {
    match response.status() {
        StatusCode::OK => Ok(Some(response)),
        StatusCode::CREATED | StatusCode::ACCEPTED | StatusCode::NO_CONTENT => Ok(None),
        _ => Err(HttpError(response.text().await.map_err(Arc::new)?)),
    }
}

#[cfg(test)]
mod tests {
    use http::response::Builder;
    use reqwest::Response;

    use crate::response_handler::handle;

    static RESPONSE: &str = "{\"message\":\"message\"}";

    #[tokio::test]
    async fn should_return_response_when_status_is_200() {
        // when
        let result = handle(response(200)).await.unwrap().unwrap();

        // then
        assert_eq!(result.status(), 200);
        assert_eq!(result.text().await.unwrap(), RESPONSE);
    }

    #[tokio::test]
    async fn should_return_none_response_when_status_is_201() {
        // when
        let result = handle(response(201)).await.unwrap();

        // then
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_return_none_response_when_status_is_204() {
        // when
        let result = handle(response(204)).await.unwrap();

        // then
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_panic_when_status_is_401() {
        // when
        let result = handle(response(401)).await.map_err(|e| e.to_string());

        // then
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), RESPONSE);
    }

    #[tokio::test]
    async fn should_panic_when_status_is_500() {
        // when
        let result = handle(response(500)).await.map_err(|e| e.to_string());

        // then
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), RESPONSE);
    }

    fn response(status: u16) -> Response {
        let response = Builder::new().status(status).body(RESPONSE).unwrap();
        Response::from(response)
    }
}
