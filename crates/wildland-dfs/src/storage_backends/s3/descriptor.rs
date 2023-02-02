use crate::storage_backends::{CloseError, OpenedFileDescriptor};

#[derive(Debug)]
pub struct S3Descriptor {}

impl OpenedFileDescriptor for S3Descriptor {
    fn close(&self) -> Result<(), CloseError> {
        Ok(())
    }

    fn read(
        &mut self,
        _count: usize,
    ) -> Result<Vec<u8>, wildland_corex::dfs::interface::DfsFrontendError> {
        todo!() // TODO COR-23
    }

    fn write(
        &mut self,
        _buf: &[u8],
    ) -> Result<usize, wildland_corex::dfs::interface::DfsFrontendError> {
        todo!() // TODO COR-23
    }

    fn seek(
        &mut self,
        _seek_from: crate::storage_backends::SeekFrom,
    ) -> Result<u64, wildland_corex::dfs::interface::DfsFrontendError> {
        todo!() // TODO COR-23
    }
}
