use crate::error::StorageControllerClientError;
use reqwest::{Response, StatusCode};

use crate::error::StorageControllerClientError::HttpError;

pub(crate) async fn handle(response: Response) -> Result<Response, StorageControllerClientError> {
    match response.status() {
        StatusCode::OK => Ok(response),
        StatusCode::CREATED => Ok(response),
        StatusCode::NO_CONTENT => Ok(response),
        StatusCode::UNAUTHORIZED => {
            log::error!("Unauthorized to make given request");
            let message = response.text().await?;
            Err(HttpError(message))
        }
        other_status => {
            let message = response.text().await?;
            log::error!("Request failed with status: {:?}", other_status);
            Err(HttpError(message))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::test_utilities::SC_RESPONSE;
    use http::response::Builder;
    use reqwest::Response;

    use crate::response_handler::handle;

    #[tokio::test]
    async fn should_return_response_when_status_is_200() {
        // when
        let result = handle(response(200)).await.unwrap();

        // then
        assert_eq!(result.status(), 200);
        assert_eq!(result.text().await.unwrap(), SC_RESPONSE);
    }

    #[tokio::test]
    async fn should_return_response_when_status_is_201() {
        // when
        let result = handle(response(201)).await.unwrap();

        // then
        assert_eq!(result.status(), 201);
        assert_eq!(result.text().await.unwrap(), SC_RESPONSE);
    }

    #[tokio::test]
    async fn should_return_response_when_status_is_204() {
        // when
        let result = handle(response(204)).await.unwrap();

        // then
        assert_eq!(result.status(), 204);
        assert_eq!(result.text().await.unwrap(), SC_RESPONSE);
    }

    #[tokio::test]
    async fn should_panic_when_status_is_401() {
        // when
        let result = handle(response(401)).await.map_err(|e| e.to_string());

        // then
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SC_RESPONSE);
    }

    #[tokio::test]
    async fn should_panic_when_status_is_500() {
        // when
        let result = handle(response(500)).await.map_err(|e| e.to_string());

        // then
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SC_RESPONSE);
    }

    fn response(status: u16) -> Response {
        let response = Builder::new().status(status).body(SC_RESPONSE).unwrap();
        Response::from(response)
    }
}
