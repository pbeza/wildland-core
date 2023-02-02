use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use aws_sdk_s3::{Client, Config, Credentials, Region};
use aws_smithy_client::erase::DynConnector;
use aws_smithy_client::http_connector::HttpConnector;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use http_body::Body as _;
use tower_service::Service;
use wildland_http_client::{Body, CurrentPlatformClient, HttpClient, HttpError};

#[derive(Clone, Default)]
struct S3Connector {
    http_client: Arc<CurrentPlatformClient>,
}

impl Service<http::Request<SdkBody>> for S3Connector {
    type Response = http::Response<SdkBody>;
    type Error = ConnectorError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<SdkBody>) -> Self::Future {
        let client = self.http_client.clone();

        Box::pin(async move {
            let (parts, mut body) = req.into_parts();

            let body = match body.data().await {
                Some(v) => Body::raw(v.map_err(ConnectorError::user)?.into_iter().collect()),
                None => Body::empty(),
            };

            let req = http::Request::from_parts(parts, body);

            client
                .send(req)
                .map(|resp| resp.map(SdkBody::from))
                .map_err(|err| match err {
                    HttpError::Io(v) => ConnectorError::io(format!("{v:?}").into()),
                    HttpError::User(v) => ConnectorError::user(format!("{v:?}").into()),
                    HttpError::Other(v) => ConnectorError::other(format!("{v:?}").into(), None),
                })
        })
    }
}

pub fn build_s3_client(credentials: Credentials, region: Region) -> Client {
    let config = Config::builder()
        .http_connector(HttpConnector::Prebuilt(Some(DynConnector::new(
            S3Connector::default(),
        ))))
        .credentials_provider(credentials)
        .region(region)
        .build();

    Client::from_conf(config)
}
