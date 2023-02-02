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

use crate::storage_backends::{CloseError, OpenedFileDescriptor, SeekFrom};

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

    fn seek(&mut self, seek_from: SeekFrom) -> Result<u64, DfsFrontendError> {
        self.inner.seek(seek_from)
    }
}
