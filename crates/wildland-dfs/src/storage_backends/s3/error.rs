use aws_sdk_s3::error::{
    CompleteMultipartUploadError,
    CreateMultipartUploadError,
    DeleteObjectError,
    GetObjectError,
    HeadObjectError,
    ListObjectsV2Error,
    PutObjectError,
    UploadPartCopyError,
    UploadPartError,
};
use aws_smithy_client::SdkError;
use thiserror::Error;
use wildland_corex::dfs::interface::DfsFrontendError;

#[derive(Error, Debug)]
pub enum S3Error {
    #[error("Not found")]
    NotFound,
    #[error("ETag mistmach")]
    ETagMistmach,
    #[error(transparent)]
    Generic(#[from] anyhow::Error),
}

impl From<S3Error> for DfsFrontendError {
    fn from(value: S3Error) -> Self {
        match value {
            S3Error::NotFound => Self::NoSuchPath,
            S3Error::ETagMistmach => Self::ConcurrentIssue,
            err @ S3Error::Generic(_) => Self::Generic(format!("{err:?}")),
        }
    }
}

macro_rules! s3_error_implement_from {
    ($Error:ty) => {
        impl From<$Error> for S3Error {
            fn from(value: $Error) -> Self {
                match value.code() {
                    Some("NotFound") => Self::NotFound,
                    Some("NoSuchKey") => Self::NotFound,
                    Some("PreconditionFailed") => Self::ETagMistmach,
                    _ => Self::Generic(value.into()),
                }
            }
        }
    };
}

s3_error_implement_from!(ListObjectsV2Error);
s3_error_implement_from!(HeadObjectError);
s3_error_implement_from!(GetObjectError);
s3_error_implement_from!(CreateMultipartUploadError);
s3_error_implement_from!(CompleteMultipartUploadError);
s3_error_implement_from!(UploadPartCopyError);
s3_error_implement_from!(UploadPartError);
s3_error_implement_from!(PutObjectError);
s3_error_implement_from!(DeleteObjectError);

impl<T> From<SdkError<T>> for S3Error
where
    T: Into<S3Error> + std::error::Error + Send + Sync + 'static,
{
    fn from(value: SdkError<T>) -> Self {
        match value {
            SdkError::ServiceError(service) => service.into_err().into(),
            _ => Self::Generic(value.into()),
        }
    }
}
