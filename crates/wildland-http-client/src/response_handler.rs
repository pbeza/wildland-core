use crate::error::WildlandHttpClientError;
use reqwest::{Response, StatusCode};

use crate::error::WildlandHttpClientError::HttpError;

#[tracing::instrument(level = "debug", ret)]
pub(crate) async fn handle(response: Response) -> Result<Response, WildlandHttpClientError> {
    match response.status() {
        StatusCode::OK => Ok(response),
        StatusCode::CREATED => Ok(response),
        StatusCode::NO_CONTENT => Ok(response),
        StatusCode::UNAUTHORIZED => {
            log::error!("Unauthorized to make given request");
            let message = response.text().await?;
            Err(HttpError(message))
        },
        StatusCode::FORBIDDEN => {
            log::error!("forbidden to make given request");
            let message = response.text().await?;
            Err(HttpError(message))
        }
        other_status => {
            log::error!("Request failed with status: {:?}", other_status);
            let message = response.text().await?;
            Err(HttpError(message))
        }
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
        let result = handle(response(200)).await.unwrap();

        // then
        assert_eq!(result.status(), 200);
        assert_eq!(result.text().await.unwrap(), RESPONSE);
    }

    #[tokio::test]
    async fn should_return_response_when_status_is_201() {
        // when
        let result = handle(response(201)).await.unwrap();

        // then
        assert_eq!(result.status(), 201);
        assert_eq!(result.text().await.unwrap(), RESPONSE);
    }

    #[tokio::test]
    async fn should_return_response_when_status_is_204() {
        // when
        let result = handle(response(204)).await.unwrap();

        // then
        assert_eq!(result.status(), 204);
        assert_eq!(result.text().await.unwrap(), RESPONSE);
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
