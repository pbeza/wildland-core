use std::ops::Neg;
use std::path::PathBuf;

use wildland_corex::dfs::interface::Stat;

use super::{CloseOnDropDescriptor, OpenedFileDescriptor};

#[derive(Debug)]
pub enum SeekFrom {
    Start { position: usize },
    End { remaining: usize },
    Current { offset: isize },
}

impl SeekFrom {
    pub fn to_std(self) -> std::io::SeekFrom {
        match self {
            SeekFrom::Start { position } => std::io::SeekFrom::Start(position as _),
            SeekFrom::End { remaining } => std::io::SeekFrom::End(
                TryInto::<i64>::try_into(remaining)
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
pub enum ReaddirResponse {
    Entries(Vec<PathBuf>),
    NotADirectory,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GetattrResponse {
    Found(Stat),
    NotFound,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateDirResponse {
    Created,
    ParentDoesNotExist,
    PathAlreadyExists,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveDirResponse {
    Removed,
    DirNotEmpty,
    NotFound,
    NotADirectory,
}
