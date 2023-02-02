use aws_sdk_s3::error::{
    HeadObjectError,
    HeadObjectErrorKind,
    ListObjectsV2Error,
    ListObjectsV2ErrorKind,
};
use aws_smithy_client::SdkError;
use thiserror::Error;

use crate::storage_backends::StorageBackendError;

#[derive(Error, Debug)]
pub enum S3Error {
    #[error("NoSuchBucket")]
    NoSuchBucket,
    #[error("NotFound")]
    NotFound,
    #[error(transparent)]
    Generic(#[from] anyhow::Error),
}

impl From<S3Error> for StorageBackendError {
    fn from(value: S3Error) -> Self {
        match value {
            err @ S3Error::NoSuchBucket => Self::Generic(err.into()),
            err @ S3Error::NotFound => Self::Generic(err.into()),
            err @ S3Error::Generic(_) => Self::Generic(err.into()),
        }
    }
}

impl From<ListObjectsV2Error> for S3Error {
    fn from(value: ListObjectsV2Error) -> Self {
        match value.kind {
            ListObjectsV2ErrorKind::NoSuchBucket(_) => Self::NoSuchBucket,
            ListObjectsV2ErrorKind::Unhandled(_) => Self::Generic(value.into()),
            _ => Self::Generic(value.into()),
        }
    }
}

impl From<HeadObjectError> for S3Error {
    fn from(value: HeadObjectError) -> Self {
        match value.kind {
            HeadObjectErrorKind::NotFound(_) => Self::NotFound,
            HeadObjectErrorKind::Unhandled(_) => Self::Generic(value.into()),
            _ => Self::Generic(value.into()),
        }
    }
}

impl<T> From<SdkError<T>> for S3Error
where
    T: Into<S3Error> + std::error::Error + Send + Sync + 'static,
{
    fn from(value: SdkError<T>) -> Self {
        match value {
            SdkError::ServiceError(service) => service.into_err().into(),
            err => Self::Generic(err.into()),
        }
    }
}
