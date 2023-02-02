use crate::storage_backends::{CloseError, OpenedFileDescriptor};

#[derive(Debug)]
pub struct S3Descriptor {}

impl OpenedFileDescriptor for S3Descriptor {
    fn close(&self) -> Result<(), CloseError> {
        Ok(())
    }
}
