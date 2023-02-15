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

use wildland_corex::dfs::interface::DfsFrontendError;

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
}
