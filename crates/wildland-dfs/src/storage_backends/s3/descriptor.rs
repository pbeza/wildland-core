use std::path::PathBuf;
use std::rc::Rc;

use derivative::Derivative;
use wildland_corex::dfs::interface::DfsFrontendError;

use super::client::S3Client;
use crate::storage_backends::models::{CloseError, SeekFrom};
use crate::storage_backends::OpenedFileDescriptor;

#[derive(thiserror::Error, Debug)]
pub enum SeekError {
    #[error("Seek out of file boundaries")]
    OutOfFileBoundaries,
}

impl From<SeekError> for DfsFrontendError {
    fn from(value: SeekError) -> Self {
        match value {
            SeekError::OutOfFileBoundaries => Self::SeekError,
        }
    }
}

#[derive(Debug)]
struct Cursor {
    total_size: usize,
    position: usize,
}

impl Cursor {
    pub fn apply_seek(&self, seek: SeekFrom) -> Result<Self, SeekError> {
        match seek {
            SeekFrom::Start { offset: position } => {
                if position as usize > self.total_size {
                    Err(SeekError::OutOfFileBoundaries)
                } else {
                    Ok(Self {
                        total_size: self.total_size,
                        position: position as usize,
                    })
                }
            }

            SeekFrom::End { offset } => {
                if offset > self.total_size as i64 {
                    Err(SeekError::OutOfFileBoundaries)
                } else {
                    self.total_size
                        .checked_add_signed(offset as isize)
                        .ok_or(SeekError::OutOfFileBoundaries)
                        .and_then(|position| {
                            if position > self.total_size {
                                Err(SeekError::OutOfFileBoundaries)
                            } else {
                                Ok(Self {
                                    total_size: self.total_size,
                                    position,
                                })
                            }
                        })
                }
            }

            SeekFrom::Current { offset } => self
                .position
                .checked_add_signed(offset as isize)
                .ok_or(SeekError::OutOfFileBoundaries)
                .and_then(|position| {
                    if position > self.total_size {
                        Err(SeekError::OutOfFileBoundaries)
                    } else {
                        Ok(Self {
                            total_size: self.total_size,
                            position,
                        })
                    }
                }),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct S3Descriptor {
    bucket_name: String,
    path: PathBuf,
    cursor: Cursor,
    etag: Option<String>,

    #[derivative(Debug = "ignore")]
    client: Rc<dyn S3Client>,
}

impl S3Descriptor {
    pub fn new(
        bucket_name: String,
        path: PathBuf,
        total_size: usize,
        etag: Option<String>,
        client: Rc<dyn S3Client>,
    ) -> Self {
        Self {
            bucket_name,
            path,
            cursor: Cursor {
                total_size,
                position: 0,
            },
            etag,
            client,
        }
    }
}

impl OpenedFileDescriptor for S3Descriptor {
    fn close(&self) -> Result<(), CloseError> {
        Ok(())
    }

    fn read(&mut self, count: usize) -> Result<Vec<u8>, DfsFrontendError> {
        let resp = self.client.read_object(
            &self.path,
            &self.bucket_name,
            self.cursor.position,
            count,
            self.etag.clone(),
        )?;
        self.cursor.position += resp.len();
        Ok(resp)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, DfsFrontendError> {
        let resp = self.client.write_buffer(
            &self.path,
            &self.bucket_name,
            self.cursor.position,
            buf,
            self.cursor.total_size,
            self.etag.clone(),
        )?;

        self.etag = resp.etag;
        self.cursor.position += resp.bytes_count;
        self.cursor.total_size = std::cmp::max(self.cursor.position, self.cursor.total_size);

        Ok(resp.bytes_count)
    }

    fn seek(&mut self, seek_from: SeekFrom) -> Result<usize, DfsFrontendError> {
        self.cursor = self.cursor.apply_seek(seek_from)?;
        Ok(self.cursor.position)
    }
}
