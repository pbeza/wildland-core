//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use wildland_corex::dfs::interface::{
    DfsFrontendError,
    FsStat,
    Stat,
    UnixTimestamp,
    WlPermissions,
};

use super::models::{CloseError, SeekFrom};

/// OpenedFileDescriptor contains state of opened file and definition of how it is stored, therefore
/// it is backend specific, cause file can be stored in different ways (e.g. partitioned depending
/// on the backend's type) and e.g. seek operation may be implemented differently.
pub trait OpenedFileDescriptor: std::fmt::Debug {
    fn close(&self) -> Result<(), CloseError>;
    /// Reads number of bytes specified by the `count` parameter and advances inner cursor of the
    /// opened file.
    ///
    /// Returns vector of bytes which can have length smaller than requested.
    fn read(&mut self, count: usize) -> Result<Vec<u8>, DfsFrontendError>;

    /// Writes bytes at the current cursor position and returns number of written bytes.
    fn write(&mut self, buf: &[u8]) -> Result<usize, DfsFrontendError>;

    /// Changes inner cursor position.
    fn seek(&mut self, seek_from: SeekFrom) -> Result<usize, DfsFrontendError>;

    /// Sets permission
    fn set_permissions(&mut self, permissions: WlPermissions) -> Result<(), DfsFrontendError>;

    /// This function will attempt to ensure that all in-memory data reaches the storage before returning.
    fn sync(&mut self) -> Result<(), DfsFrontendError>;

    /// Queries metadata about the underlying file.
    fn metadata(&mut self) -> Result<Stat, DfsFrontendError>;

    /// Sets access and modification time.
    ///
    /// Passing None as an argument means not overwriting given parameter (not setting it to None)
    fn set_times(
        &mut self,
        access_time: Option<UnixTimestamp>,
        modification_time: Option<UnixTimestamp>,
    ) -> Result<(), DfsFrontendError>;

    /// Truncates or extends the underlying file, updating the size of this file to become `length`.
    /// If the size is less than the current file’s size, then the file will be shrunk. If it is greater
    /// than the current file’s size, then the file will be extended to size and have all of the intermediate
    /// data filled in with 0s.
    /// If the file’s cursor was further than the new length then the file is
    /// shrunk using this operation, the cursor will now be at the new end of file.
    fn set_length(&mut self, length: usize) -> Result<(), DfsFrontendError>;

    /// Returns information about a mounted filesystem containing the `file`.
    fn stat_fs(&mut self) -> Result<FsStat, DfsFrontendError>;
}

/// Wrapper ensuring that close is always called on `OpenedFileDescriptor`
#[derive(Debug)]
pub struct CloseOnDropDescriptor {
    inner: Box<dyn OpenedFileDescriptor>,
}

impl CloseOnDropDescriptor {
    pub fn new(inner: Box<dyn OpenedFileDescriptor>) -> Self {
        Self { inner }
    }
}

impl Drop for CloseOnDropDescriptor {
    fn drop(&mut self) {
        let _ = self.inner.close();
    }
}

impl OpenedFileDescriptor for CloseOnDropDescriptor {
    fn close(&self) -> Result<(), CloseError> {
        self.inner.close()
    }

    fn read(&mut self, count: usize) -> Result<Vec<u8>, DfsFrontendError> {
        self.inner.read(count)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, DfsFrontendError> {
        self.inner.write(buf)
    }

    fn seek(&mut self, seek_from: SeekFrom) -> Result<usize, DfsFrontendError> {
        self.inner.seek(seek_from)
    }

    fn set_permissions(&mut self, permissions: WlPermissions) -> Result<(), DfsFrontendError> {
        self.inner.set_permissions(permissions)
    }

    fn sync(&mut self) -> Result<(), DfsFrontendError> {
        self.inner.sync()
    }

    fn metadata(&mut self) -> Result<Stat, DfsFrontendError> {
        self.inner.metadata()
    }

    fn set_times(
        &mut self,
        access_time: Option<UnixTimestamp>,
        modification_time: Option<UnixTimestamp>,
    ) -> Result<(), DfsFrontendError> {
        self.inner.set_times(access_time, modification_time)
    }

    fn set_length(&mut self, length: usize) -> Result<(), DfsFrontendError> {
        self.inner.set_length(length)
    }

    fn stat_fs(&mut self) -> Result<FsStat, DfsFrontendError> {
        self.inner.stat_fs()
    }
}
