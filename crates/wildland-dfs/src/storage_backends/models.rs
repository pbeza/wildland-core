use std::ops::Neg;
use std::path::PathBuf;

use wildland_corex::dfs::interface::{FsStat, Stat};

use super::{CloseOnDropDescriptor, OpenedFileDescriptor};

#[derive(Debug)]
pub enum SeekFrom {
    Start { offset: u64 },
    End { offset: i64 },
    Current { offset: i64 },
}

impl SeekFrom {
    pub fn to_std(self) -> std::io::SeekFrom {
        match self {
            SeekFrom::Start { offset } => std::io::SeekFrom::Start(offset as _),
            SeekFrom::End { offset } => std::io::SeekFrom::End(
                TryInto::<i64>::try_into(offset)
                    .map(|v| v.neg())
                    .unwrap_or(i64::MIN),
            ),
            SeekFrom::Current { offset } => std::io::SeekFrom::Current(offset as _),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CloseError {
    #[error("File has been already closed")]
    FileAlreadyClosed,
}

#[derive(thiserror::Error, Debug)]
pub enum StorageBackendError {
    #[error(transparent)]
    Generic(#[from] anyhow::Error),
}

impl From<std::io::Error> for StorageBackendError {
    fn from(e: std::io::Error) -> Self {
        Self::Generic(e.into())
    }
}

impl From<std::path::StripPrefixError> for StorageBackendError {
    fn from(e: std::path::StripPrefixError) -> Self {
        Self::Generic(e.into())
    }
}

#[derive(Debug)]
pub enum OpenResponse {
    Found(CloseOnDropDescriptor),
    NotAFile,
    NotFound,
}

impl OpenResponse {
    pub fn found<T: OpenedFileDescriptor + 'static>(descriptor: T) -> Self {
        Self::Found(CloseOnDropDescriptor::new(Box::new(descriptor)))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReadDirResponse {
    Entries(Vec<PathBuf>),
    NoSuchPath,
    NotADirectory,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MetadataResponse {
    Found(Stat),
    NotFound,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateDirResponse {
    Created,
    InvalidParent,
    PathAlreadyExists,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveDirResponse {
    Removed,
    DirNotEmpty,
    NotFound,
    NotADirectory,
    RootRemovalNotAllowed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveFileResponse {
    Removed,
    NotFound,
    NotAFile,
}

#[derive(Debug)]
pub enum RenameResponse {
    Renamed,
    NotFound,
    SourceIsParentOfTarget,
    TargetPathAlreadyExists,
}

#[derive(Debug)]
pub enum CreateFileResponse {
    Created(CloseOnDropDescriptor),
    ParentDoesNotExist,
}

impl CreateFileResponse {
    pub fn created<T: OpenedFileDescriptor + 'static>(descriptor: T) -> Self {
        Self::Created(CloseOnDropDescriptor::new(Box::new(descriptor)))
    }
}

#[derive(Debug)]
pub enum SetPermissionsResponse {
    Set,
    NotFound,
}

#[derive(Debug)]
pub enum StatFsResponse {
    Stat(FsStat),
    NotFound,
    NotSupported(String),
}
