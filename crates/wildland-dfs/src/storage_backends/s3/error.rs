use aws_sdk_s3::error::{
    CompleteMultipartUploadError,
    CompleteMultipartUploadErrorKind,
    CreateMultipartUploadError,
    CreateMultipartUploadErrorKind,
    GetObjectError,
    GetObjectErrorKind,
    HeadObjectError,
    HeadObjectErrorKind,
    ListObjectsV2Error,
    ListObjectsV2ErrorKind,
    UploadPartCopyError,
    UploadPartCopyErrorKind,
    UploadPartError,
    UploadPartErrorKind,
};
use aws_smithy_client::SdkError;
use thiserror::Error;
use wildland_corex::dfs::interface::DfsFrontendError;

#[derive(Error, Debug)]
pub enum S3Error {
    #[error("No such bucket")]
    NoSuchBucket,
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
            err @ S3Error::NoSuchBucket => Self::Generic(err.to_string()),
            S3Error::NotFound => Self::NoSuchPath,
            S3Error::ETagMistmach => Self::ConcurrentIssue,
            err @ S3Error::Generic(_) => Self::Generic(format!("{err:?}")),
        }
    }
}

impl From<ListObjectsV2Error> for S3Error {
    fn from(value: ListObjectsV2Error) -> Self {
        if let Some("PreconditionFailed") = value.code() {
            return Self::ETagMistmach;
        }

        match value.kind {
            ListObjectsV2ErrorKind::NoSuchBucket(_) => Self::NoSuchBucket,
            ListObjectsV2ErrorKind::Unhandled(_) => Self::Generic(value.into()),
            _ => Self::Generic(value.into()),
        }
    }
}

impl From<HeadObjectError> for S3Error {
    fn from(value: HeadObjectError) -> Self {
        if let Some("PreconditionFailed") = value.code() {
            return Self::ETagMistmach;
        }

        match value.kind {
            HeadObjectErrorKind::NotFound(_) => Self::NotFound,
            HeadObjectErrorKind::Unhandled(_) => Self::Generic(value.into()),
            _ => Self::Generic(value.into()),
        }
    }
}

impl From<GetObjectError> for S3Error {
    fn from(value: GetObjectError) -> Self {
        if let Some("PreconditionFailed") = value.code() {
            return Self::ETagMistmach;
        }

        match value.kind {
            GetObjectErrorKind::NoSuchKey(_) => Self::NotFound,
            GetObjectErrorKind::InvalidObjectState(_) => Self::Generic(value.into()),
            GetObjectErrorKind::Unhandled(_) => Self::Generic(value.into()),
            _ => Self::Generic(value.into()),
        }
    }
}

impl From<CreateMultipartUploadError> for S3Error {
    fn from(value: CreateMultipartUploadError) -> Self {
        if let Some("PreconditionFailed") = value.code() {
            return Self::ETagMistmach;
        }

        match value.kind {
            CreateMultipartUploadErrorKind::Unhandled(_) => Self::Generic(value.into()),
            _ => Self::Generic(value.into()),
        }
    }
}

impl From<CompleteMultipartUploadError> for S3Error {
    fn from(value: CompleteMultipartUploadError) -> Self {
        if let Some("PreconditionFailed") = value.code() {
            return Self::ETagMistmach;
        }

        match value.kind {
            CompleteMultipartUploadErrorKind::Unhandled(_) => Self::Generic(value.into()),
            _ => Self::Generic(value.into()),
        }
    }
}

impl From<UploadPartCopyError> for S3Error {
    fn from(value: UploadPartCopyError) -> Self {
        if let Some("PreconditionFailed") = value.code() {
            return Self::ETagMistmach;
        }

        match value.kind {
            UploadPartCopyErrorKind::Unhandled(_) => Self::Generic(value.into()),
            _ => Self::Generic(value.into()),
        }
    }
}

impl From<UploadPartError> for S3Error {
    fn from(value: UploadPartError) -> Self {
        if let Some("PreconditionFailed") = value.code() {
            return Self::ETagMistmach;
        }

        match value.kind {
            UploadPartErrorKind::Unhandled(_) => Self::Generic(value.into()),
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
