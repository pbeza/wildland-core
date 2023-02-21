use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::rc::Rc;

use derivative::Derivative;
use wildland_corex::dfs::interface::{
    DfsFrontendError,
    FsStat,
    Stat,
    UnixTimestamp,
    WlPermissions,
};

use super::client::S3Client;
use super::file_system::FileSystemNodeRef;
use super::helpers::{commit_file_system, load_file_system};
use super::models::WriteResp;
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
    object_name: String,
    node_path: PathBuf,
    cursor: Cursor,
    e_tag: String,

    #[derivative(Debug = "ignore")]
    client: Rc<dyn S3Client>,
}

impl S3Descriptor {
    pub fn new(
        bucket_name: String,
        object_name: String,
        node_path: PathBuf,
        total_size: usize,
        e_tag: String,
        client: Rc<dyn S3Client>,
    ) -> Self {
        Self {
            bucket_name,
            object_name,
            node_path,
            cursor: Cursor {
                total_size,
                position: 0,
            },
            e_tag,
            client,
        }
    }
}

impl OpenedFileDescriptor for S3Descriptor {
    fn close(&self) -> Result<(), CloseError> {
        Ok(())
    }

    fn read(&mut self, count: usize) -> Result<Vec<u8>, DfsFrontendError> {
        if self.cursor.position == self.cursor.total_size {
            return Ok(Vec::new());
        }

        let resp = self.client.read_object(
            &self.object_name,
            &self.bucket_name,
            Some(RangeInclusive::new(self.cursor.position, count - 1)),
            Some(self.e_tag.clone()),
        )?;
        self.cursor.position += resp.len();
        Ok(resp)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, DfsFrontendError> {
        let mut file_system = load_file_system(&*self.client, &self.bucket_name)
            .map_err(|err| DfsFrontendError::Generic(format!("{err:?}")))?;

        let WriteResp {
            bytes_count,
            new_object_name,
            new_modification_time,
            new_e_tag,
        } = self.client.write_buffer(
            &self.object_name,
            &self.bucket_name,
            self.cursor.position,
            buf,
            self.cursor.total_size,
            Some(self.e_tag.clone()),
        )?;

        let new_position = self.cursor.position + bytes_count;
        let new_total_size = std::cmp::max(new_position, self.cursor.total_size);

        match file_system.get_node(&self.node_path) {
            Some(FileSystemNodeRef::File(file)) => {
                file.object_name = new_object_name.clone();
                file.size = new_total_size;
                file.e_tag = new_e_tag.clone();
                file.modification_time = new_modification_time;
            }
            _ => return Err(DfsFrontendError::ConcurrentIssue),
        };

        commit_file_system(&*self.client, &self.bucket_name, file_system)
            .map_err(|err| DfsFrontendError::Generic(format!("{err:?}")))?;

        let _ = self
            .client
            .remove_object(&self.object_name, &self.bucket_name);

        self.object_name = new_object_name;
        self.e_tag = new_e_tag;
        self.cursor.position = new_position;
        self.cursor.total_size = new_total_size;

        Ok(bytes_count)
    }

    fn seek(&mut self, seek_from: SeekFrom) -> Result<usize, DfsFrontendError> {
        self.cursor = self.cursor.apply_seek(seek_from)?;
        Ok(self.cursor.position)
    }

    fn set_permissions(&mut self, _permissions: WlPermissions) -> Result<(), DfsFrontendError> {
        todo!() // TODO COR-87
    }

    fn sync(&mut self) -> Result<(), DfsFrontendError> {
        todo!() // TODO COR-87
    }

    fn metadata(&mut self) -> Result<Stat, DfsFrontendError> {
        todo!() // TODO COR-87
    }

    fn set_times(
        &mut self,
        _access_time: Option<UnixTimestamp>,
        _modification_time: Option<UnixTimestamp>,
    ) -> Result<(), DfsFrontendError> {
        todo!() // TODO COR-87
    }

    fn set_length(&mut self, _length: usize) -> Result<(), DfsFrontendError> {
        todo!() // TODO COR-87
    }

    fn stat_fs(&mut self) -> Result<FsStat, DfsFrontendError> {
        todo!() // TODO COR-87
    }
}
